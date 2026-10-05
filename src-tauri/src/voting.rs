#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use base64::Engine;
use rand::Rng;
use rusqlite::{params, OptionalExtension, Row};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::db::Database;
use crate::metadata::{self, Track};
use crate::log::{Subsystem::Voting, Level::*};

pub const DAILY_LIMIT: i64 = 3;
pub const COOKIE: &str = "__Host-zv";
pub const TOKEN_BYTES: usize = 32;
pub const INACTIVE_SECS: i64 = 180 * 24 * 60 * 60;
pub const BOARD_SIZE: usize = 6;
const MINE_WINDOW_SECS: i64 = 14 * 24 * 60 * 60;
const TURNSTILE_VERIFY: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";
const TURNSTILE_ACTION: &str = "device";
const TURNSTILE_PROBE: &str = "XXXX.DUMMY.TOKEN.XXXX";
const SEEN_GRANULARITY: i64 = 60 * 60;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pending,
    Approved,
    Added,
    Rejected,
}

impl Status {
    pub fn AsText(self) -> &'static str {
        match self {
            Status::Pending => "pending",
            Status::Approved => "approved",
            Status::Added => "added",
            Status::Rejected => "rejected",
        }
    }

    pub fn Parse(Text: &str) -> Option<Self> {
        match Text {
            "pending" => Some(Status::Pending),
            "approved" => Some(Status::Approved),
            "added" => Some(Status::Added),
            "rejected" => Some(Status::Rejected),
            _ => None,
        }
    }
}

pub struct Device {
    pub Id: String,
    pub Banned: bool,
    pub Created: i64,
}

pub struct Limiter {
    Windows: Mutex<HashMap<String, (u32, Instant)>>,
}

impl Limiter {
    fn New() -> Self {
        Self { Windows: Mutex::new(HashMap::new()) }
    }

    pub fn Hit(&self, Key: &str, Max: u32, Window: Duration) -> Result<(), u64> {
        let mut Held = self.Windows.lock().unwrap();
        let Now = Instant::now();

        if Held.len() > 50_000 {
            Held.retain(|_, (_, Start)| Now.duration_since(*Start) < Duration::from_secs(24 * 60 * 60));
        }

        let Entry = Held.entry(Key.to_string()).or_insert((0, Now));
        if Now.duration_since(Entry.1) >= Window {
            *Entry = (0, Now);
        }

        if Entry.0 >= Max {
            let Remaining = Window.saturating_sub(Now.duration_since(Entry.1));
            return Err(Remaining.as_secs().max(1));
        }

        Entry.0 += 1;
        Ok(())
    }
}

const LATENCY_SAMPLES: usize = 256;

#[derive(Default)]
pub struct Activity {
    Searches: AtomicU64,
    Failures: AtomicU64,
    Votes: AtomicU64,
    Requests: AtomicU64,
    Limited: AtomicU64,
    Latency: Mutex<VecDeque<u32>>,
}

impl Activity {
    pub fn Searched(&self, Took: Duration, Succeeded: bool) {
        self.Searches.fetch_add(1, Ordering::Relaxed);
        if !Succeeded {
            self.Failures.fetch_add(1, Ordering::Relaxed);
            return;
        }
        let mut Samples = self.Latency.lock().unwrap();
        if Samples.len() == LATENCY_SAMPLES {
            Samples.pop_front();
        }
        Samples.push_back(Took.as_millis().min(u32::MAX as u128) as u32);
    }

    pub fn Voted(&self) {
        self.Votes.fetch_add(1, Ordering::Relaxed);
    }

    pub fn Requested(&self) {
        self.Requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn Throttled(&self) {
        self.Limited.fetch_add(1, Ordering::Relaxed);
    }

    pub fn Json(&self, Db: &Database) -> Value {
        let mut Sorted: Vec<u32> = self.Latency.lock().unwrap().iter().copied().collect();
        Sorted.sort_unstable();
        let Average = (!Sorted.is_empty())
            .then(|| (Sorted.iter().map(|V| *V as u64).sum::<u64>() / Sorted.len() as u64) as u32);
        let Slow = (!Sorted.is_empty())
            .then(|| Sorted[((Sorted.len() * 95).div_ceil(100)).saturating_sub(1)]);

        let (Active, Devices) = Db.Lock().query_row(
            "SELECT COALESCE(SUM(seen >= ?1 AND banned = 0), 0), COUNT(*) FROM vote_devices",
            [Now() - 24 * 60 * 60],
            |R| Ok((R.get::<_, i64>(0)?, R.get::<_, i64>(1)?)),
        ).unwrap_or((0, 0));

        json!({
            "Searches": self.Searches.load(Ordering::Relaxed),
            "Failures": self.Failures.load(Ordering::Relaxed),
            "Votes": self.Votes.load(Ordering::Relaxed),
            "Requests": self.Requests.load(Ordering::Relaxed),
            "Limited": self.Limited.load(Ordering::Relaxed),
            "Average": Average,
            "Slow": Slow,
            "Active": Active,
            "Devices": Devices,
        })
    }
}

pub struct Runtime {
    pub Activity: Activity,
    pub Limits: Limiter,
    Ephemeral: [u8; 32],
    Pepper: OnceLock<Vec<u8>>,
    BoardPending: AtomicBool,
}

impl Runtime {
    pub fn New() -> Self {
        let mut Ephemeral = [0u8; 32];
        rand::rand_core::UnwrapErr(rand::rngs::SysRng).fill_bytes(&mut Ephemeral);
        Self {
            Activity: Activity::default(),
            Limits: Limiter::New(),
            Ephemeral,
            Pepper: OnceLock::new(),
            BoardPending: AtomicBool::new(false),
        }
    }

    fn Pepper(&self, Db: &Database) -> &[u8] {
        self.Pepper.get_or_init(|| Db.VotePepper())
    }

    pub fn NetworkKey(&self, Ip: &str) -> String {
        hex::encode(&Hmac(&self.Ephemeral, Ip.as_bytes())[..16])
    }

    pub fn DeviceId(&self, Db: &Database, Token: &str) -> String {
        hex::encode(Hmac(self.Pepper(Db), Token.as_bytes()))
    }

    pub fn ClaimBoardUpdate(&self) -> bool {
        !self.BoardPending.swap(true, Ordering::AcqRel)
    }

    pub fn ReleaseBoardUpdate(&self) {
        self.BoardPending.store(false, Ordering::Release);
    }
}

pub fn Hmac(Key: &[u8], Message: &[u8]) -> [u8; 32] {
    const BLOCK: usize = 64;
    let mut Padded = [0u8; BLOCK];
    if Key.len() > BLOCK {
        Padded[..32].copy_from_slice(&Sha256::digest(Key));
    } else {
        Padded[..Key.len()].copy_from_slice(Key);
    }

    let mut Inner = Sha256::new();
    Inner.update(Padded.map(|B| B ^ 0x36));
    Inner.update(Message);
    let InnerHash = Inner.finalize();

    let mut Outer = Sha256::new();
    Outer.update(Padded.map(|B| B ^ 0x5c));
    Outer.update(InnerHash);
    Outer.finalize().into()
}

pub fn Now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|D| D.as_secs() as i64).unwrap_or(0)
}

pub fn NewToken() -> String {
    let mut Bytes = [0u8; TOKEN_BYTES];
    rand::rand_core::UnwrapErr(rand::rngs::SysRng).fill_bytes(&mut Bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Bytes)
}

pub fn ValidToken(Token: &str) -> bool {
    Token.len() == 43 && Token.bytes().all(|B| B.is_ascii_alphanumeric() || B == b'-' || B == b'_')
}

pub fn Handle(Id: &str) -> String {
    let Short: Vec<char> = Id.chars().take(12).collect();
    Short.chunks(4).map(|C| C.iter().collect::<String>()).collect::<Vec<String>>().join("·")
}

pub fn ParseHandle(Text: &str) -> Option<String> {
    let Clean: String = Text
        .chars()
        .filter(|C| !matches!(C, '·' | '-' | ' ' | '.' | ':'))
        .collect::<String>()
        .to_lowercase();
    if Clean.len() < 8 || Clean.len() > 64 || !Clean.bytes().all(|B| B.is_ascii_hexdigit()) {
        return None;
    }
    Some(Clean)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SecretCheck {
    Valid,
    Invalid,
    Unreachable,
}

impl SecretCheck {
    pub fn AsText(self) -> &'static str {
        match self {
            SecretCheck::Valid => "valid",
            SecretCheck::Invalid => "invalid",
            SecretCheck::Unreachable => "unreachable",
        }
    }
}

pub async fn CheckSecret(Secret: &str) -> SecretCheck {
    let Client = match reqwest::Client::builder().timeout(Duration::from_secs(8)).build() {
        Ok(C) => C,
        Err(E) => {
            fail!(Voting, E, "could not build turnstile client");
            return SecretCheck::Unreachable;
        }
    };

    let Sent = Client
        .post(TURNSTILE_VERIFY)
        .form(&[("secret", Secret), ("response", TURNSTILE_PROBE)])
        .send()
        .await;

    let Text = match Sent {
        Ok(Res) => match Res.text().await {
            Ok(T) => T,
            Err(E) => {
                fail!(Voting, E, "turnstile check returned an unreadable body");
                return SecretCheck::Unreachable;
            }
        },
        Err(E) => {
            fail!(Voting, E, "turnstile check could not reach cloudflare");
            return SecretCheck::Unreachable;
        }
    };

    let Body: Value = match serde_json::from_str(&Text) {
        Ok(B) => B,
        Err(E) => {
            fail!(Voting, E, "turnstile check returned malformed json");
            return SecretCheck::Unreachable;
        }
    };

    let Rejected = Body["error-codes"].as_array().is_some_and(|Codes| {
        Codes.iter().any(|C| matches!(C.as_str(), Some("invalid-input-secret" | "missing-input-secret")))
    });

    if Rejected { SecretCheck::Invalid } else { SecretCheck::Valid }
}

pub async fn VerifyCaptcha(Db: &Database, Response: &str, Ip: Option<&str>) -> Result<(), String> {
    let Secret = Db.GetTurnstileSecret();
    if Secret.is_empty() {
        return Err("unconfigured".into());
    }
    if Response.is_empty() || Response.len() > 2048 {
        return Err("malformed token".into());
    }

    let mut Idempotency = [0u8; 16];
    rand::rand_core::UnwrapErr(rand::rngs::SysRng).fill_bytes(&mut Idempotency);
    let IdempotencyKey = {
        let H = hex::encode(Idempotency);
        format!("{}-{}-4{}-a{}-{}", &H[0..8], &H[8..12], &H[13..16], &H[17..20], &H[20..32])
    };

    let mut Form: Vec<(&str, &str)> = vec![
        ("secret", Secret.as_str()),
        ("response", Response),
        ("idempotency_key", IdempotencyKey.as_str()),
    ];
    if let Some(Address) = Ip {
        Form.push(("remoteip", Address));
    }

    let Client = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .map_err(|E| E.to_string())?;

    let Text = Client
        .post(TURNSTILE_VERIFY)
        .form(&Form)
        .send()
        .await
        .map_err(|E| E.to_string())?
        .text()
        .await
        .map_err(|E| E.to_string())?;
    let Body: Value = serde_json::from_str(&Text).map_err(|E| E.to_string())?;

    if !Body["success"].as_bool().unwrap_or(false) {
        return Err(format!("rejected: {}", Body["error-codes"]));
    }

    if Body["action"].as_str() != Some(TURNSTILE_ACTION) {
        return Err(format!("wrong action: {}", Body["action"]));
    }

    let Expected = crate::cloudflared::NormalizeHostname(&Db.GetVotingHostname());
    let Actual = Body["hostname"].as_str().unwrap_or("").to_lowercase();
    if !Expected.is_empty() && Actual != Expected {
        return Err(format!("hostname mismatch: {}", Actual));
    }

    Ok(())
}

pub fn CreateDevice(Db: &Database, Id: &str) {
    let At = Now();
    Db.Lock().execute(
        "INSERT OR IGNORE INTO vote_devices (id, created, seen) VALUES (?1, ?2, ?2)",
        params![Id, At],
    ).ok();
}

pub fn FindDevice(Db: &Database, Id: &str) -> Option<Device> {
    let Conn = Db.Lock();
    let Found = Conn.query_row(
        "SELECT id, banned, created, seen FROM vote_devices WHERE id = ?1",
        [Id],
        |R| Ok((Device { Id: R.get(0)?, Banned: R.get::<_, i64>(1)? == 1, Created: R.get(2)? }, R.get::<_, i64>(3)?)),
    ).optional().ok().flatten()?;

    let (Found, Seen) = Found;
    let At = Now();
    if At - Seen > SEEN_GRANULARITY {
        Conn.execute("UPDATE vote_devices SET seen = ?2 WHERE id = ?1", params![Id, At]).ok();
    }
    Some(Found)
}

pub fn RequestsToday(Db: &Database, Id: &str) -> i64 {
    Db.Lock().query_row(
        "SELECT COUNT(*) FROM vote_songs WHERE device = ?1 AND day = date('now', 'localtime')",
        [Id],
        |R| R.get(0),
    ).unwrap_or(0)
}

pub fn DeviceJson(Db: &Database, D: &Device) -> Value {
    json!({
        "Handle": Handle(&D.Id),
        "Banned": D.Banned,
        "Left": (DAILY_LIMIT - RequestsToday(Db, &D.Id)).max(0),
        "Since": D.Created,
    })
}

fn _Art(Key: Option<String>, Prefix: &str) -> Option<String> {
    Key.map(|K| format!("{}/{}", Prefix, K))
}

pub fn TrackJson(Db: &Database, T: &Track) -> Value {
    let ArtKey = T.Cover.as_deref().and_then(|Url| metadata::RegisterArt(Db, Url));
    let Status: Option<String> = Db.Lock().query_row(
        "SELECT status FROM vote_songs WHERE norm = ?1",
        [metadata::Key(&T.Title, &T.Artist)],
        |R| R.get(0),
    ).optional().ok().flatten();

    json!({
        "Provider": T.Provider.AsText(),
        "Id": T.Id,
        "Title": T.Title,
        "Artist": T.Artist,
        "Album": T.Album,
        "Cover": _Art(ArtKey, "/api/art"),
        "Explicit": T.Explicit,
        "DurationMs": T.DurationMs,
        "Status": Status,
    })
}

pub enum RequestOutcome {
    Created(Value),
    Duplicate(String),
    Limit,
}

fn _MySong(R: &Row) -> rusqlite::Result<Value> {
    Ok(json!({
        "Id": R.get::<_, i64>(0)?,
        "Title": R.get::<_, String>(1)?,
        "Artist": R.get::<_, String>(2)?,
        "Album": R.get::<_, Option<String>>(3)?,
        "Cover": _Art(R.get::<_, Option<String>>(4)?, "/api/art"),
        "Explicit": R.get::<_, Option<i64>>(5)?.map(|E| E == 1),
        "Status": R.get::<_, String>(6)?,
        "Created": R.get::<_, i64>(7)?,
    }))
}

pub fn CreateRequest(Db: &Database, DeviceId: &str, T: &Track) -> RequestOutcome {
    let Norm = metadata::Key(&T.Title, &T.Artist);
    let ArtKey = T.Cover.as_deref().and_then(|Url| metadata::RegisterArt(Db, Url));
    let mut Conn = Db.Lock();
    let Tx = match Conn.transaction() {
        Ok(Tx) => Tx,
        Err(E) => {
            log!(Voting, Error, "transaction failed: {}", E);
            return RequestOutcome::Limit;
        }
    };

    let Used: i64 = Tx.query_row(
        "SELECT COUNT(*) FROM vote_songs WHERE device = ?1 AND day = date('now', 'localtime')",
        [DeviceId],
        |R| R.get(0),
    ).unwrap_or(DAILY_LIMIT);
    if Used >= DAILY_LIMIT {
        return RequestOutcome::Limit;
    }

    let Existing: Option<String> = Tx.query_row(
        "SELECT status FROM vote_songs WHERE norm = ?1",
        [&Norm],
        |R| R.get(0),
    ).optional().ok().flatten();
    if let Some(Status) = Existing {
        return RequestOutcome::Duplicate(Status);
    }

    let At = Now();
    let Inserted = Tx.execute(
        "INSERT INTO vote_songs
            (norm, provider, ext_id, title, artist, album, cover_url, art_key, explicit, duration_ms, link, status, device, day, created)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'pending', ?12, date('now', 'localtime'), ?13)",
        params![
            Norm,
            T.Provider.AsText(),
            T.Id,
            T.Title,
            T.Artist,
            T.Album,
            T.Cover,
            ArtKey,
            T.Explicit.map(|E| E as i64),
            T.DurationMs,
            T.Link,
            DeviceId,
            At,
        ],
    );

    if let Err(E) = Inserted {
        log!(Voting, Error, "insert request failed: {}", E);
        return RequestOutcome::Duplicate("pending".into());
    }

    let Id = Tx.last_insert_rowid();
    let Song = Tx.query_row(
        "SELECT id, title, artist, album, art_key, explicit, status, created FROM vote_songs WHERE id = ?1",
        [Id],
        _MySong,
    ).unwrap_or(Value::Null);

    if Tx.commit().is_err() {
        return RequestOutcome::Limit;
    }

    RequestOutcome::Created(Song)
}

const SCORE_SQL: &str = "1 + COALESCE((SELECT SUM(b.value) FROM vote_ballots b JOIN vote_devices d ON d.id = b.device WHERE b.song = s.id AND d.banned = 0), 0)";

pub fn Board(Db: &Database, DeviceId: Option<&str>) -> Value {
    let Conn = Db.Lock();
    let Sql = format!(
        "SELECT s.id, s.title, s.artist, s.album, s.art_key, s.explicit, {SCORE_SQL} AS score,
                COALESCE((SELECT value FROM vote_ballots WHERE song = s.id AND device = ?1), 0)
         FROM vote_songs s WHERE s.status = 'approved'
         ORDER BY score DESC, s.decided ASC, s.id ASC LIMIT 200"
    );

    let Board: Vec<Value> = Conn.prepare(&Sql).and_then(|mut Stmt| {
        Stmt.query_map([DeviceId.unwrap_or("")], |R| Ok(json!({
            "Id": R.get::<_, i64>(0)?,
            "Title": R.get::<_, String>(1)?,
            "Artist": R.get::<_, String>(2)?,
            "Album": R.get::<_, Option<String>>(3)?,
            "Cover": _Art(R.get::<_, Option<String>>(4)?, "/api/art"),
            "Explicit": R.get::<_, Option<i64>>(5)?.map(|E| E == 1),
            "Score": R.get::<_, i64>(6)?,
            "Vote": R.get::<_, i64>(7)?,
        })))?.collect()
    }).unwrap_or_default();

    let Mine: Vec<Value> = match DeviceId {
        Some(Id) => Conn.prepare(
            "SELECT id, title, artist, album, art_key, explicit, status, created FROM vote_songs
             WHERE device = ?1 AND created >= ?2 ORDER BY created DESC LIMIT 50",
        ).and_then(|mut Stmt| Stmt.query_map(params![Id, Now() - MINE_WINDOW_SECS], _MySong)?.collect())
            .unwrap_or_default(),
        None => Vec::new(),
    };

    json!({ "Board": Board, "Mine": Mine })
}

pub fn Vote(Db: &Database, DeviceId: &str, Song: i64, Value: i64) -> Option<(i64, i64)> {
    let Conn = Db.Lock();
    let Approved: bool = Conn.query_row(
        "SELECT status = 'approved' FROM vote_songs WHERE id = ?1",
        [Song],
        |R| R.get(0),
    ).optional().ok().flatten().unwrap_or(false);
    if !Approved {
        return None;
    }

    let Changed = if Value == 0 {
        Conn.execute("DELETE FROM vote_ballots WHERE song = ?1 AND device = ?2", params![Song, DeviceId])
    } else {
        Conn.execute(
            "INSERT INTO vote_ballots (song, device, value, at) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(song, device) DO UPDATE SET value = ?3, at = ?4",
            params![Song, DeviceId, Value, Now()],
        )
    };
    if let Err(E) = Changed {
        log!(Voting, Error, "ballot write failed: {}", E);
        return None;
    }

    let Score: i64 = Conn.query_row(
        &format!("SELECT {SCORE_SQL} FROM vote_songs s WHERE s.id = ?1"),
        [Song],
        |R| R.get(0),
    ).unwrap_or(1);

    Some((Score, Value))
}

pub fn TvBoard(Db: &Database) -> Value {
    let Enabled = Db.IsVotingEnabled();
    let Host = crate::cloudflared::NormalizeHostname(&Db.GetVotingHostname());
    let Url = (!Host.is_empty()).then(|| format!("https://{}", Host));

    let Songs: Vec<Value> = Db.Lock().prepare(&format!(
        "SELECT s.id, s.title, s.artist, s.cover_url, {SCORE_SQL} AS score, s.explicit
         FROM vote_songs s WHERE s.status = 'approved'
         ORDER BY score DESC, s.decided ASC, s.id ASC LIMIT {BOARD_SIZE}"
    )).and_then(|mut Stmt| Stmt.query_map([], |R| Ok(json!({
        "Id": R.get::<_, i64>(0)?,
        "Title": R.get::<_, String>(1)?,
        "Artist": R.get::<_, String>(2)?,
        "Cover": R.get::<_, Option<String>>(3)?,
        "Score": R.get::<_, i64>(4)?,
        "Explicit": R.get::<_, Option<i64>>(5)?.map(|E| E == 1),
    })))?.collect()).unwrap_or_default();

    json!({ "Enabled": Enabled, "Url": Url, "Songs": Songs })
}

fn _AdminSong(R: &Row) -> rusqlite::Result<Value> {
    let Device: Option<String> = R.get(13)?;
    Ok(json!({
        "Id": R.get::<_, i64>(0)?,
        "Provider": R.get::<_, String>(1)?,
        "Title": R.get::<_, String>(2)?,
        "Artist": R.get::<_, String>(3)?,
        "Album": R.get::<_, Option<String>>(4)?,
        "Cover": _Art(R.get::<_, Option<String>>(5)?, "/api/voting/art"),
        "Explicit": R.get::<_, Option<i64>>(6)?.map(|E| E == 1),
        "DurationMs": R.get::<_, Option<i64>>(7)?,
        "Link": R.get::<_, Option<String>>(8)?,
        "Status": R.get::<_, String>(9)?,
        "Score": R.get::<_, i64>(10)?,
        "Up": R.get::<_, i64>(11)?,
        "Down": R.get::<_, i64>(12)?,
        "Device": Device.as_deref().map(Handle),
        "DeviceBanned": R.get::<_, Option<i64>>(14)?.is_some_and(|B| B == 1),
        "Created": R.get::<_, i64>(15)?,
        "Decided": R.get::<_, Option<i64>>(16)?,
    }))
}

fn _AdminSelect() -> String {
    format!(
        "SELECT s.id, s.provider, s.title, s.artist, s.album, s.art_key, s.explicit, s.duration_ms, s.link, s.status,
                {SCORE_SQL} AS score,
                (SELECT COUNT(*) FROM vote_ballots b JOIN vote_devices d ON d.id = b.device WHERE b.song = s.id AND b.value = 1 AND d.banned = 0),
                (SELECT COUNT(*) FROM vote_ballots b JOIN vote_devices d ON d.id = b.device WHERE b.song = s.id AND b.value = -1 AND d.banned = 0),
                s.device, (SELECT banned FROM vote_devices WHERE id = s.device), s.created, s.decided
         FROM vote_songs s"
    )
}

pub fn AdminSongs(Db: &Database, Status: Status) -> Value {
    let Order = match Status {
        Status::Pending => "s.created ASC",
        Status::Approved => "score DESC, s.decided ASC",
        Status::Added | Status::Rejected => "s.decided DESC",
    };
    let Sql = format!("{} WHERE s.status = ?1 ORDER BY {} LIMIT 500", _AdminSelect(), Order);
    let List: Vec<Value> = Db.Lock()
        .prepare(&Sql)
        .and_then(|mut Stmt| Stmt.query_map([Status.AsText()], _AdminSong)?.collect())
        .unwrap_or_default();
    Value::Array(List)
}

pub fn AdminSong(Db: &Database, Id: i64) -> Option<Value> {
    let Sql = format!("{} WHERE s.id = ?1", _AdminSelect());
    Db.Lock().query_row(&Sql, [Id], _AdminSong).optional().ok().flatten()
}

pub fn SetStatus(Db: &Database, Id: i64, Next: Status) -> bool {
    let Decided = (Next != Status::Pending).then(Now);
    Db.Lock().execute(
        "UPDATE vote_songs SET status = ?2, decided = ?3 WHERE id = ?1",
        params![Id, Next.AsText(), Decided],
    ).map(|N| N > 0).unwrap_or(false)
}

pub fn DeleteSong(Db: &Database, Id: i64) -> bool {
    Db.Lock().execute("DELETE FROM vote_songs WHERE id = ?1", [Id]).map(|N| N > 0).unwrap_or(false)
}

pub fn Counts(Db: &Database) -> Value {
    let Conn = Db.Lock();
    let mut Counts: HashMap<String, i64> = HashMap::new();
    if let Ok(mut Stmt) = Conn.prepare("SELECT status, COUNT(*) FROM vote_songs GROUP BY status") {
        if let Ok(Rows) = Stmt.query_map([], |R| Ok((R.get::<_, String>(0)?, R.get::<_, i64>(1)?))) {
            for (Status, Count) in Rows.flatten() {
                Counts.insert(Status, Count);
            }
        }
    }
    let Banned: i64 = Conn.query_row("SELECT COUNT(*) FROM vote_devices WHERE banned = 1", [], |R| R.get(0)).unwrap_or(0);

    json!({
        "Counts": {
            "Pending": Counts.get("pending").copied().unwrap_or(0),
            "Approved": Counts.get("approved").copied().unwrap_or(0),
            "Added": Counts.get("added").copied().unwrap_or(0),
            "Rejected": Counts.get("rejected").copied().unwrap_or(0),
        },
        "Banned": Banned,
    })
}

fn _BannedJson(R: &Row) -> rusqlite::Result<Value> {
    Ok(json!({
        "Handle": Handle(&R.get::<_, String>(0)?),
        "Reason": R.get::<_, Option<String>>(1)?,
        "BannedAt": R.get::<_, Option<i64>>(2)?.unwrap_or(0),
        "Created": R.get::<_, i64>(3)?,
        "Seen": R.get::<_, i64>(4)?,
        "Requests": R.get::<_, i64>(5)?,
    }))
}

const DEVICE_SELECT: &str = "SELECT d.id, d.ban_reason, d.banned_at, d.created, d.seen,
    (SELECT COUNT(*) FROM vote_songs WHERE device = d.id) FROM vote_devices d";

pub fn BannedDevices(Db: &Database) -> Value {
    let Sql = format!("{} WHERE d.banned = 1 ORDER BY d.banned_at DESC LIMIT 500", DEVICE_SELECT);
    let List: Vec<Value> = Db.Lock()
        .prepare(&Sql)
        .and_then(|mut Stmt| Stmt.query_map([], _BannedJson)?.collect())
        .unwrap_or_default();
    Value::Array(List)
}

pub enum BanOutcome {
    Done(Value),
    NotFound,
    Ambiguous,
}

pub fn SetBan(Db: &Database, Prefix: &str, Banned: bool, Reason: Option<&str>) -> BanOutcome {
    let Conn = Db.Lock();
    let Pattern = format!("{}%", Prefix);
    let Matches: Vec<String> = Conn
        .prepare("SELECT id FROM vote_devices WHERE id LIKE ?1 LIMIT 2")
        .and_then(|mut Stmt| Stmt.query_map([&Pattern], |R| R.get(0))?.collect())
        .unwrap_or_default();

    let Id = match Matches.as_slice() {
        [] => return BanOutcome::NotFound,
        [One] => One.clone(),
        _ => return BanOutcome::Ambiguous,
    };

    let Reason = Reason.map(str::trim).filter(|R| !R.is_empty()).map(|R| R.chars().take(200).collect::<String>());
    if Banned {
        Conn.execute(
            "UPDATE vote_devices SET banned = 1, ban_reason = ?2, banned_at = ?3 WHERE id = ?1",
            params![Id, Reason, Now()],
        ).ok();
        Conn.execute(
            "UPDATE vote_songs SET status = 'rejected', decided = ?2 WHERE device = ?1 AND status = 'pending'",
            params![Id, Now()],
        ).ok();
    } else {
        Conn.execute(
            "UPDATE vote_devices SET banned = 0, ban_reason = NULL, banned_at = NULL WHERE id = ?1",
            [&Id],
        ).ok();
    }

    let Sql = format!("{} WHERE d.id = ?1", DEVICE_SELECT);
    match Conn.query_row(&Sql, [&Id], _BannedJson) {
        Ok(Found) => BanOutcome::Done(Found),
        Err(_) => BanOutcome::NotFound,
    }
}

pub fn ArtUrl(Db: &Database, Key: &str) -> Option<String> {
    Db.Lock().query_row(
        "SELECT cover_url FROM vote_songs WHERE art_key = ?1 LIMIT 1",
        [Key],
        |R| R.get(0),
    ).optional().ok().flatten()
}

pub fn SecondsToReset(Db: &Database) -> i64 {
    Db.Lock().query_row(
        "SELECT CAST(strftime('%s', date('now', 'localtime', '+1 day')) AS INTEGER)
              - CAST(strftime('%s', 'now', 'localtime') AS INTEGER)",
        [],
        |R| R.get::<_, i64>(0),
    ).unwrap_or(0).max(0)
}

pub fn PreviewSource(Db: &Database, Id: i64, Approved: bool) -> Option<(metadata::Provider, String, String, String)> {
    let (Provider, ExtId, Title, Artist): (String, String, String, String) = Db.Lock().query_row(
        "SELECT provider, ext_id, title, artist FROM vote_songs WHERE id = ?1 AND (?2 = 0 OR status = 'approved')",
        rusqlite::params![Id, Approved],
        |R| Ok((R.get(0)?, R.get(1)?, R.get(2)?, R.get(3)?)),
    ).optional().ok().flatten()?;
    Some((metadata::Provider::Parse(&Provider).unwrap_or(metadata::Provider::Lastfm), ExtId, Title, Artist))
}

pub fn Purge(Db: &Database) {
    let Cutoff = Now() - INACTIVE_SECS;
    match Db.Lock().execute("DELETE FROM vote_devices WHERE banned = 0 AND seen < ?1", [Cutoff]) {
        Ok(0) => {}
        Ok(N) => log!(Voting, Info, "purged {} inactive devices", N),
        Err(E) => log!(Voting, Error, "purge failed: {}", E),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hmac_matches_rfc4231_case_2() {
        let Out = Hmac(b"Jefe", b"what do ya want for nothing?");
        assert_eq!(hex::encode(Out), "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843");
    }

    #[test]
    fn hmac_handles_long_keys_rfc4231_case_6() {
        let Key = [0xaau8; 131];
        let Out = Hmac(&Key, b"Test Using Larger Than Block-Size Key - Hash Key First");
        assert_eq!(hex::encode(Out), "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54");
    }

    #[test]
    fn tokens_are_well_formed() {
        let T = NewToken();
        assert!(ValidToken(&T));
        assert!(!ValidToken("short"));
        assert!(!ValidToken(&format!("{}=", &T[..42])));
    }

    #[test]
    fn handles_roundtrip() {
        let Id = "a3f9c21e07bd00112233445566778899aabbccddeeff00112233445566778899";
        let Shown = Handle(Id);
        assert_eq!(Shown, "a3f9·c21e·07bd");
        assert_eq!(ParseHandle(&Shown).as_deref(), Some("a3f9c21e07bd"));
        assert_eq!(ParseHandle("A3F9-C21E"), Some("a3f9c21e".into()));
        assert_eq!(ParseHandle("zz"), None);
        assert_eq!(ParseHandle("a3f9%"), None);
    }

    #[test]
    fn limiter_blocks_after_budget() {
        let L = Limiter::New();
        for _ in 0..3 {
            assert!(L.Hit("k", 3, Duration::from_secs(60)).is_ok());
        }
        assert!(L.Hit("k", 3, Duration::from_secs(60)).is_err());
        assert!(L.Hit("other", 3, Duration::from_secs(60)).is_ok());
    }

    fn _Db() -> (Database, std::path::PathBuf) {
        let Path = std::env::temp_dir().join(format!("zstio-vote-test-{}.db", NewToken()));
        (Database::Open(&Path), Path)
    }

    fn _Track(Title: &str) -> Track {
        Track {
            Provider: metadata::Provider::Deezer,
            Id: "1".into(),
            Title: Title.into(),
            Artist: "Artist".into(),
            Album: None,
            Cover: None,
            Explicit: Some(false),
            DurationMs: None,
            Link: None,
        }
    }

    #[test]
    fn requests_respect_limit_duplicates_and_scores() {
        let (Db, Path) = _Db();
        CreateDevice(&Db, "aa");
        CreateDevice(&Db, "bb");

        assert!(matches!(CreateRequest(&Db, "aa", &_Track("One")), RequestOutcome::Created(_)));
        assert!(matches!(CreateRequest(&Db, "bb", &_Track("One (Remastered)")), RequestOutcome::Duplicate(S) if S == "pending"));
        assert!(matches!(CreateRequest(&Db, "aa", &_Track("Two")), RequestOutcome::Created(_)));
        assert!(matches!(CreateRequest(&Db, "aa", &_Track("Three")), RequestOutcome::Created(_)));
        assert!(matches!(CreateRequest(&Db, "aa", &_Track("Four")), RequestOutcome::Limit));

        let Id: i64 = Db.Lock().query_row("SELECT id FROM vote_songs WHERE title = 'One'", [], |R| R.get(0)).unwrap();
        assert!(Vote(&Db, "bb", Id, 1).is_none());
        assert!(SetStatus(&Db, Id, Status::Approved));
        assert_eq!(Vote(&Db, "bb", Id, 1), Some((2, 1)));
        assert_eq!(Vote(&Db, "aa", Id, -1), Some((1, -1)));
        assert_eq!(Vote(&Db, "aa", Id, 0), Some((2, 0)));

        assert!(matches!(SetBan(&Db, "bb", true, Some("spam")), BanOutcome::Done(_)));
        assert_eq!(Vote(&Db, "aa", Id, 0), Some((1, 0)));
        assert!(matches!(SetBan(&Db, "cc", true, None), BanOutcome::NotFound));

        drop(Db);
        std::fs::remove_file(&Path).ok();
    }
}
