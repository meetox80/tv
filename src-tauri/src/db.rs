#![allow(non_snake_case)]

use std::path::Path;
use std::sync::Mutex;
use rusqlite::Connection;
use sha2::{Sha256, Digest};
use crate::log::{Subsystem::Core, Level::*};

pub const DEFAULT_LATITUDE: f64 = 50.04;
pub const DEFAULT_LONGITUDE: f64 = 22.62;
pub const SCREEN_PAGES: [&str; 3] = ["lesson", "news", "voting"];
pub const PAGE_SECONDS: std::ops::RangeInclusive<u16> = 5..=600;
const DEFAULT_PAGE_SECONDS: u16 = 30;

pub type LastPlayed = (String, String, Option<(Vec<u8>, String)>);

pub struct Cached {
    pub Value: Option<String>,
    pub Bytes: Option<Vec<u8>>,
}

fn _Now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|D| D.as_secs() as i64)
        .unwrap_or(0)
}

pub const SLIDES_PREFIX: &str = "slides:";

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ScreenPage {
    pub Id: String,
    pub Enabled: bool,
    pub Seconds: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub Label: Option<String>,
}

pub struct Database {
    Conn: Mutex<Connection>,
}

impl Database {
    pub fn Open(Path: &Path) -> Self {
        if let Some(Parent) = Path.parent() {
            std::fs::create_dir_all(Parent).ok();
        }

        let Conn = Connection::open(Path).expect("failed to open database");
        log!(Core, Info, "database: {:?}", Path);

        Conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS config (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS vote_devices (
                id         TEXT PRIMARY KEY,
                created    INTEGER NOT NULL,
                seen       INTEGER NOT NULL,
                banned     INTEGER NOT NULL DEFAULT 0,
                ban_reason TEXT,
                banned_at  INTEGER
            );

            CREATE TABLE IF NOT EXISTS vote_songs (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                norm        TEXT NOT NULL UNIQUE,
                provider    TEXT NOT NULL,
                ext_id      TEXT NOT NULL,
                title       TEXT NOT NULL,
                artist      TEXT NOT NULL,
                album       TEXT,
                cover_url   TEXT,
                art_key     TEXT,
                explicit    INTEGER,
                duration_ms INTEGER,
                link        TEXT,
                status      TEXT NOT NULL CHECK (status IN ('pending', 'approved', 'added', 'rejected')),
                device      TEXT REFERENCES vote_devices(id) ON DELETE SET NULL,
                day         TEXT NOT NULL,
                created     INTEGER NOT NULL,
                decided     INTEGER
            );

            CREATE INDEX IF NOT EXISTS vote_songs_status ON vote_songs(status);
            CREATE INDEX IF NOT EXISTS vote_songs_device_day ON vote_songs(device, day);
            CREATE INDEX IF NOT EXISTS vote_songs_art ON vote_songs(art_key);

            CREATE TABLE IF NOT EXISTS vote_ballots (
                song   INTEGER NOT NULL REFERENCES vote_songs(id) ON DELETE CASCADE,
                device TEXT NOT NULL REFERENCES vote_devices(id) ON DELETE CASCADE,
                value  INTEGER NOT NULL CHECK (value IN (-1, 1)),
                at     INTEGER NOT NULL,
                PRIMARY KEY (song, device)
            );

            CREATE TABLE IF NOT EXISTS slide_collections (
                id       TEXT PRIMARY KEY,
                name     TEXT NOT NULL,
                position INTEGER NOT NULL DEFAULT 0,
                slides   TEXT NOT NULL DEFAULT '[]',
                created  INTEGER NOT NULL,
                updated  INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS slide_assets (
                id      TEXT PRIMARY KEY,
                mime    TEXT NOT NULL,
                bytes   BLOB NOT NULL,
                created INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS cache (
                kind    TEXT NOT NULL,
                key     TEXT NOT NULL,
                value   TEXT,
                bytes   BLOB,
                expires INTEGER NOT NULL,
                touched INTEGER NOT NULL,
                PRIMARY KEY (kind, key)
            );

            CREATE INDEX IF NOT EXISTS cache_expires ON cache(expires);
            CREATE INDEX IF NOT EXISTS cache_touched ON cache(touched);

            CREATE TABLE IF NOT EXISTS last_played (
                id     INTEGER PRIMARY KEY CHECK (id = 1),
                title  TEXT NOT NULL,
                artist TEXT NOT NULL,
                cover  BLOB,
                mime   TEXT,
                at     INTEGER NOT NULL
            );
        ").expect("failed to initialize database");

        Self { Conn: Mutex::new(Conn) }
    }

    pub fn Lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.Conn.lock().unwrap()
    }

    pub fn Get(&self, Key: &str) -> Option<String> {
        let Conn = self.Conn.lock().unwrap();
        Conn.query_row(
            "SELECT value FROM config WHERE key = ?1",
            [Key],
            |Row| Row.get(0),
        ).ok()
    }

    pub fn Set(&self, Key: &str, Value: &str) {
        let Conn = self.Conn.lock().unwrap();
        Conn.execute(
            "INSERT INTO config (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
            [Key, Value],
        ).ok();
    }

    pub fn CacheGet(&self, Kind: &str, Key: &str) -> Option<Cached> {
        let Conn = self.Conn.lock().unwrap();
        Conn.query_row(
            "SELECT value, bytes FROM cache WHERE kind = ?1 AND key = ?2 AND expires > ?3",
            rusqlite::params![Kind, Key, _Now()],
            |R| Ok(Cached { Value: R.get(0)?, Bytes: R.get(1)? }),
        ).ok()
    }

    pub fn CachePut(&self, Kind: &str, Key: &str, Value: Option<&str>, Bytes: Option<&[u8]>, Ttl: std::time::Duration) {
        let Now = _Now();
        let Expires = Now.saturating_add(Ttl.as_secs().min(i64::MAX as u64 / 2) as i64);
        let Conn = self.Conn.lock().unwrap();
        if let Err(E) = Conn.execute(
            "INSERT INTO cache (kind, key, value, bytes, expires, touched) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(kind, key) DO UPDATE SET value = ?3, bytes = ?4, expires = ?5, touched = ?6",
            rusqlite::params![Kind, Key, Value, Bytes, Expires, Now],
        ) {
            fail!(Core, E, "could not write {} cache entry", Kind);
        }
    }

    pub fn CachePrune(&self, ByteBudget: i64) -> usize {
        let Conn = self.Conn.lock().unwrap();
        let mut Removed = Conn.execute("DELETE FROM cache WHERE expires <= ?1", [_Now()]).unwrap_or(0);
        loop {
            let Used: i64 = Conn
                .query_row("SELECT COALESCE(SUM(length(bytes)), 0) FROM cache", [], |R| R.get(0))
                .unwrap_or(0);
            if Used <= ByteBudget {
                break;
            }
            let Dropped = Conn.execute(
                "DELETE FROM cache WHERE rowid IN (SELECT rowid FROM cache WHERE bytes IS NOT NULL ORDER BY touched ASC LIMIT 32)",
                [],
            ).unwrap_or(0);
            if Dropped == 0 {
                break;
            }
            Removed += Dropped;
        }
        Removed
    }

    pub fn SaveLastPlayed(&self, Title: &str, Artist: &str) {
        let Conn = self.Conn.lock().unwrap();
        if let Err(E) = Conn.execute(
            "INSERT INTO last_played (id, title, artist, cover, mime, at)
             VALUES (1, ?1, ?2, NULL, NULL, CAST(strftime('%s', 'now') AS INTEGER))
             ON CONFLICT(id) DO UPDATE SET title = ?1, artist = ?2, cover = NULL, mime = NULL, at = excluded.at",
            [Title, Artist],
        ) {
            fail!(Core, E, "could not save last played track");
        }
    }

    pub fn SaveLastCover(&self, Bytes: &[u8], Mime: &str) {
        let Conn = self.Conn.lock().unwrap();
        if let Err(E) = Conn.execute(
            "UPDATE last_played SET cover = ?1, mime = ?2 WHERE id = 1",
            rusqlite::params![Bytes, Mime],
        ) {
            fail!(Core, E, "could not save last played cover");
        }
    }

    pub fn LastPlayed(&self) -> Option<LastPlayed> {
        let Conn = self.Conn.lock().unwrap();
        Conn.query_row(
            "SELECT title, artist, cover, mime FROM last_played WHERE id = 1",
            [],
            |R| {
                let Cover: Option<Vec<u8>> = R.get(2)?;
                let Mime: Option<String> = R.get(3)?;
                Ok((R.get(0)?, R.get(1)?, Cover.zip(Mime)))
            },
        ).ok()
    }

    pub fn SetPassword(&self, Plain: &str) {
        let Hash = hex::encode(Sha256::digest(Plain.as_bytes()));
        self.Set("admin_password", &Hash);
    }

    pub fn CheckPassword(&self, Plain: &str) -> bool {
        let Hash = hex::encode(Sha256::digest(Plain.as_bytes()));
        self.Get("admin_password").map_or(false, |Stored| Stored == Hash)
    }

    pub fn IsSetupComplete(&self) -> bool {
        self.Get("setup_complete").map_or(false, |V| V == "1")
    }

    pub fn MarkSetupComplete(&self) {
        self.Set("setup_complete", "1");
        log!(Core, Info, "setup marked complete");
    }

    pub fn GetTunnelToken(&self) -> String {
        self.Get("tunnel_token").unwrap_or_default()
    }

    pub fn GetManagementHostname(&self) -> String {
        self.Get("management_hostname").unwrap_or_default()
    }

    pub fn GetVotingHostname(&self) -> String {
        self.Get("voting_hostname").unwrap_or_default()
    }

    pub fn GetApiKey(&self) -> String {
        match self.Get("scrobble_api_key") {
            Some(Key) => Key,
            None => {
                let Key = Self::GenerateHexKey();
                self.Set("scrobble_api_key", &Key);
                Key
            }
        }
    }

    pub fn RegenerateApiKey(&self) -> String {
        let Key = Self::GenerateHexKey();
        self.Set("scrobble_api_key", &Key);
        Key
    }

    pub fn GetScrobbleMethod(&self) -> String {
        self.Get("scrobble_method").unwrap_or_default()
    }

    pub fn SetScrobbleMethod(&self, Method: &str) {
        self.Set("scrobble_method", Method);
    }

    pub fn IsScrobbleInLessons(&self) -> bool {
        self.Get("scrobble_in_lessons").map_or(true, |V| V == "1")
    }

    pub fn SetScrobbleInLessons(&self, Enabled: bool) {
        self.Set("scrobble_in_lessons", if Enabled { "1" } else { "0" });
    }

    pub fn GetLastfmApiKey(&self) -> String {
        self.Get("lastfm_api_key").unwrap_or_default()
    }

    pub fn SetLastfmApiKey(&self, Key: &str) {
        self.Set("lastfm_api_key", Key);
    }

    pub fn GetLastfmUsername(&self) -> String {
        self.Get("lastfm_username").unwrap_or_default()
    }

    pub fn SetLastfmUsername(&self, Username: &str) {
        self.Set("lastfm_username", Username);
    }

    pub fn IsLastfmVerified(&self) -> bool {
        self.Get("lastfm_verified").map_or(false, |V| V == "1")
    }

    pub fn SetLastfmVerified(&self, Verified: bool) {
        self.Set("lastfm_verified", if Verified { "1" } else { "0" });
    }

    pub fn GetSpotifyClientId(&self) -> String {
        self.Get("spotify_client_id").unwrap_or_default()
    }

    pub fn GetSpotifyClientSecret(&self) -> String {
        self.Get("spotify_client_secret").unwrap_or_default()
    }

    pub fn SetSpotifyCredentials(&self, ClientId: &str, ClientSecret: &str) {
        self.Set("spotify_client_id", ClientId);
        self.Set("spotify_client_secret", ClientSecret);
    }

    pub fn GetSpotifyRefreshToken(&self) -> String {
        self.Get("spotify_refresh_token").unwrap_or_default()
    }

    pub fn SetSpotifyRefreshToken(&self, Token: &str) {
        self.Set("spotify_refresh_token", Token);
    }

    pub fn IsSpotifyConnected(&self) -> bool {
        !self.GetSpotifyRefreshToken().is_empty()
    }

    pub fn TakeSpotifyOauthState(&self) -> String {
        let Stored = self.Get("spotify_oauth_state").unwrap_or_default();
        self.Set("spotify_oauth_state", "");
        Stored
    }

    pub fn NewSpotifyOauthState(&self) -> String {
        let State = Self::GenerateHexKey();
        self.Set("spotify_oauth_state", &State);
        State
    }

    pub fn DisconnectSpotify(&self) {
        self.Set("spotify_refresh_token", "");
        self.Set("spotify_oauth_state", "");
    }

    pub fn GetWeatherEnabled(&self) -> bool {
        self.Get("screen_weather_enabled").map_or(false, |V| V == "1")
    }

    pub fn GetWeatherLatitude(&self) -> f64 {
        self.Get("screen_weather_lat")
            .and_then(|V| V.parse().ok())
            .unwrap_or(DEFAULT_LATITUDE)
    }

    pub fn GetWeatherLongitude(&self) -> f64 {
        self.Get("screen_weather_lon")
            .and_then(|V| V.parse().ok())
            .unwrap_or(DEFAULT_LONGITUDE)
    }

    pub fn GetSerialPort(&self) -> String {
        self.Get("serial_port").unwrap_or_default()
    }

    pub fn GetSerialBaud(&self) -> u32 {
        self.Get("serial_baud")
            .and_then(|V| V.parse().ok())
            .unwrap_or(crate::serial::DEFAULT_BAUD)
    }

    pub fn SetSerial(&self, Port: &str, Baud: u32) {
        self.Set("serial_port", Port);
        self.Set("serial_baud", &Baud.to_string());
    }

    pub fn GetSerialSecretKey(&self) -> String {
        self.Get("serial_secret_key").unwrap_or_default()
    }

    pub fn SetSerialSecretKey(&self, Key: &str) {
        self.Set("serial_secret_key", Key);
    }

    pub fn GetSerialPeerKey(&self) -> String {
        self.Get("serial_peer_key").unwrap_or_default()
    }

    pub fn SetSerialPeerKey(&self, Key: &str) {
        self.Set("serial_peer_key", Key);
    }

    pub fn GetSerialLastSeq(&self) -> u64 {
        self.Get("serial_last_seq").and_then(|V| V.parse().ok()).unwrap_or(0)
    }

    pub fn SetSerialLastSeq(&self, Seq: u64) {
        self.Set("serial_last_seq", &Seq.to_string());
    }

    pub fn GetScreenNotice(&self) -> String {
        self.Get("screen_notice").unwrap_or_default()
    }

    pub fn SetScreenNotice(&self, Text: &str) {
        self.Set("screen_notice", Text);
    }

    pub fn ScreenPageIds(&self) -> Vec<(String, Option<String>)> {
        let mut Ids: Vec<(String, Option<String>)> = SCREEN_PAGES.iter().map(|Id| (Id.to_string(), None)).collect();
        let Conn = self.Conn.lock().unwrap();
        let Collections = Conn
            .prepare("SELECT id, name FROM slide_collections ORDER BY position, created")
            .and_then(|mut Stmt| {
                Stmt.query_map([], |R| Ok((R.get::<_, String>(0)?, R.get::<_, String>(1)?)))?
                    .collect::<Result<Vec<_>, _>>()
            })
            .unwrap_or_default();
        Ids.extend(Collections.into_iter().map(|(Id, Name)| (format!("{}{}", SLIDES_PREFIX, Id), Some(Name))));
        Ids
    }

    pub fn GetScreenPages(&self) -> Vec<ScreenPage> {
        let Known = self.ScreenPageIds();
        let Stored: Vec<ScreenPage> = self.Get("screen_pages")
            .and_then(|Raw| serde_json::from_str(&Raw).ok())
            .unwrap_or_default();

        let mut Pages: Vec<ScreenPage> = Vec::with_capacity(Known.len());
        for Page in Stored {
            let Some((_, Label)) = Known.iter().find(|(Id, _)| *Id == Page.Id) else { continue };
            if Pages.iter().any(|P| P.Id == Page.Id) {
                continue;
            }
            let Seconds = Page.Seconds.clamp(*PAGE_SECONDS.start(), *PAGE_SECONDS.end());
            Pages.push(ScreenPage { Seconds, Label: Label.clone(), ..Page });
        }
        for (Id, Label) in Known {
            if !Pages.iter().any(|P| P.Id == Id) {
                Pages.push(ScreenPage { Id, Enabled: true, Seconds: DEFAULT_PAGE_SECONDS, Label });
            }
        }
        Pages
    }

    pub fn SetScreenPages(&self, Pages: &[ScreenPage]) {
        let Pages: Vec<ScreenPage> = Pages.iter().map(|P| ScreenPage { Label: None, ..P.clone() }).collect();
        match serde_json::to_string(&Pages) {
            Ok(Raw) => self.Set("screen_pages", &Raw),
            Err(E) => { fail!(Core, E, "could not encode screen pages"); }
        }
    }

    pub fn GetLessonDuration(&self) -> u8 {
        match self.Get("screen_lesson_duration").as_deref() {
            Some("30") => 30,
            _ => 45,
        }
    }

    pub fn SetLessonDuration(&self, Minutes: u8) {
        self.Set("screen_lesson_duration", &Minutes.to_string());
    }

    pub fn SetWeather(&self, Enabled: bool, Latitude: f64, Longitude: f64) {
        self.Set("screen_weather_enabled", if Enabled { "1" } else { "0" });
        self.Set("screen_weather_lat", &Latitude.to_string());
        self.Set("screen_weather_lon", &Longitude.to_string());
    }

    pub fn IsVotingEnabled(&self) -> bool {
        self.Get("voting_enabled").is_some_and(|V| V == "1")
    }

    pub fn SetVotingEnabled(&self, Enabled: bool) {
        self.Set("voting_enabled", if Enabled { "1" } else { "0" });
    }

    pub fn GetTurnstileSiteKey(&self) -> String {
        self.Get("turnstile_site_key").unwrap_or_default()
    }

    pub fn GetTurnstileSecret(&self) -> String {
        self.Get("turnstile_secret").unwrap_or_default()
    }

    pub fn SetTurnstile(&self, SiteKey: &str, Secret: Option<&str>) {
        self.Set("turnstile_site_key", SiteKey);
        if let Some(Value) = Secret {
            self.Set("turnstile_secret", Value);
        }
    }

    pub fn ClearTurnstile(&self) {
        self.Set("turnstile_site_key", "");
        self.Set("turnstile_secret", "");
        self.SetVotingEnabled(false);
    }

    pub fn VotePepper(&self) -> Vec<u8> {
        if let Some(Stored) = self.Get("vote_pepper").and_then(|H| hex::decode(H).ok()).filter(|B| B.len() == 32) {
            return Stored;
        }
        use rand::Rng;
        let mut Fresh = vec![0u8; 32];
        rand::rand_core::UnwrapErr(rand::rngs::SysRng).fill_bytes(&mut Fresh);
        self.Set("vote_pepper", &hex::encode(&Fresh));
        Fresh
    }

    fn GenerateHexKey() -> String {
        use rand::RngExt;
        let mut Rng = rand::rng();
        let Bytes: Vec<u8> = (0..16).map(|_| Rng.random()).collect();
        hex::encode(Bytes)
    }

    pub fn Nuke(&self) {
        let Conn = self.Conn.lock().unwrap();
        Conn.execute_batch("
            DELETE FROM vote_ballots;
            DELETE FROM vote_songs;
            DELETE FROM vote_devices;
            DELETE FROM last_played;
            DELETE FROM cache;
            DELETE FROM slide_collections;
            DELETE FROM slide_assets;
            DELETE FROM config;
        ").ok();
        log!(Core, Info, "database nuked");
    }
}
