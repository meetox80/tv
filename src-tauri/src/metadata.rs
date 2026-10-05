#![allow(non_snake_case)]

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use base64::Engine;
use futures_util::future::join_all;
use sha2::{Digest, Sha256};

use crate::cover;
use crate::db::Database;
use crate::log::{Subsystem::Metadata, Level::*};

const SPOTIFY_API: &str = "https://api.spotify.com/v1";
const LASTFM_API: &str = "https://ws.audioscrobbler.com/2.0/";
const DEEZER_API: &str = "https://api.deezer.com";
const ITUNES_API: &str = "https://itunes.apple.com/search";
const SPOTIFY_EMBED: &str = "https://open.spotify.com/embed/track";
const SPOTIFY_TOKEN_TRACK: &str = "4cOdK2wGLETKBW3PvgPWqT";
const SPOTIFY_PATHFINDER: &str = "https://api-partner.spotify.com/pathfinder/v2/query";
const SPOTIFY_SEARCH_HASH: &str = "1148393611bbc58e84e47aed35ecc731275df9f9eb660956962e352dd3631d89";
const BROWSER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0 Safari/537.36";
const LASTFM_PLACEHOLDER: &str = "2a96cbd8b46e442fc41c2b86b821562f";
const MARKET: &str = "PL";
const RESULT_LIMIT: usize = 12;
const ENRICH_BUDGET: Duration = Duration::from_secs(5);
const HOUR: u64 = 60 * 60;
const TRACK_TTL: Duration = Duration::from_secs(7 * 24 * HOUR);
const TRACK_PARTIAL_TTL: Duration = Duration::from_secs(HOUR);
const QUERY_TTL: Duration = Duration::from_secs(12 * HOUR);
const QUERY_PARTIAL_TTL: Duration = Duration::from_secs(15 * 60);
const ART_STORE_TTL: Duration = Duration::from_secs(30 * 24 * HOUR);
const PREVIEW_TTL: Duration = Duration::from_secs(24 * HOUR);
const ART_TTL: Duration = Duration::from_secs(6 * HOUR);
const ART_BUDGET: usize = 48 * 1024 * 1024;
const ART_HOSTS: [&str; 7] = [
    "i.scdn.co",
    "mosaic.scdn.co",
    ".spotifycdn.com",
    ".dzcdn.net",
    "lastfm.freetls.fastly.net",
    ".mzstatic.com",
    "lastfm-img2.akamaized.net",
];

const PREVIEW_HOSTS: [&str; 1] = ["p.scdn.co"];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Spotify,
    Lastfm,
    Deezer,
}

impl Provider {
    pub fn AsText(self) -> &'static str {
        match self {
            Provider::Spotify => "spotify",
            Provider::Lastfm => "lastfm",
            Provider::Deezer => "deezer",
        }
    }

    pub fn Parse(Text: &str) -> Option<Self> {
        match Text {
            "spotify" => Some(Provider::Spotify),
            "lastfm" => Some(Provider::Lastfm),
            "deezer" => Some(Provider::Deezer),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Track {
    pub Provider: Provider,
    pub Id: String,
    pub Title: String,
    pub Artist: String,
    pub Album: Option<String>,
    pub Cover: Option<String>,
    pub Explicit: Option<bool>,
    pub DurationMs: Option<u32>,
    pub Link: Option<String>,
}

impl Track {
    fn Complete(&self) -> bool {
        self.Cover.is_some() && self.Explicit.is_some() && self.Album.is_some() && self.DurationMs.is_some()
    }

    fn Absorb(&mut self, Other: Found) {
        if self.Cover.is_none() { self.Cover = Other.Cover; }
        if self.Explicit.is_none() { self.Explicit = Other.Explicit; }
        if self.Album.is_none() { self.Album = Other.Album; }
        if self.DurationMs.is_none() { self.DurationMs = Other.DurationMs; }
    }
}

#[derive(Default)]
struct Found {
    Cover: Option<String>,
    Explicit: Option<bool>,
    Album: Option<String>,
    DurationMs: Option<u32>,
}

impl From<Track> for Found {
    fn from(T: Track) -> Self {
        Found { Cover: T.Cover, Explicit: T.Explicit, Album: T.Album, DurationMs: T.DurationMs }
    }
}

struct SpotifyToken {
    Value: String,
    ExpiresAt: Instant,
    ForClient: String,
}

struct Art {
    Bytes: Vec<u8>,
    Mime: String,
    At: Instant,
}

#[derive(Default)]
struct Registry {
    ArtBytes: HashMap<String, Art>,
    ArtUsed: usize,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Answer {
    Source: Provider,
    Tracks: Vec<Track>,
}

fn _Registry() -> &'static Mutex<Registry> {
    static R: OnceLock<Mutex<Registry>> = OnceLock::new();
    R.get_or_init(|| Mutex::new(Registry::default()))
}

fn _Token() -> &'static Mutex<Option<SpotifyToken>> {
    static T: OnceLock<Mutex<Option<SpotifyToken>>> = OnceLock::new();
    T.get_or_init(|| Mutex::new(None))
}

fn _Anon() -> &'static Mutex<Option<(String, Instant)>> {
    static A: OnceLock<Mutex<Option<(String, Instant)>>> = OnceLock::new();
    A.get_or_init(|| Mutex::new(None))
}

fn _Client() -> &'static reqwest::Client {
    static C: OnceLock<reqwest::Client> = OnceLock::new();
    C.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(6))
            .connect_timeout(Duration::from_secs(4))
            .redirect(reqwest::redirect::Policy::limited(3))
            .user_agent(concat!("zstio-tv/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("metadata http client")
    })
}

fn _ArtClient() -> &'static reqwest::Client {
    static C: OnceLock<reqwest::Client> = OnceLock::new();
    C.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .connect_timeout(Duration::from_secs(4))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(concat!("zstio-tv/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("art http client")
    })
}

pub fn Normalize(Text: &str) -> String {
    let Lower = Text.to_lowercase();
    let mut Cut = String::with_capacity(Lower.len());
    let mut Depth = 0i32;

    for C in Lower.chars() {
        match C {
            '(' | '[' | '{' => Depth += 1,
            ')' | ']' | '}' => Depth = (Depth - 1).max(0),
            _ if Depth == 0 => Cut.push(C),
            _ => {}
        }
    }

    let Cut = Cut.split(" - ").next().unwrap_or("").to_string();
    let Cut = [" feat. ", " feat ", " ft. ", " ft ", " featuring ", " with "]
        .iter()
        .fold(Cut, |Acc, Marker| Acc.split(Marker).next().unwrap_or("").to_string());

    Cut.chars()
        .map(|C| if C.is_alphanumeric() { C } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

fn _LeadArtist(Artist: &str) -> String {
    let First = Artist
        .split([',', '&', '/', ';'])
        .next()
        .unwrap_or(Artist);
    let First = [" feat", " ft.", " x ", " and ", " i "]
        .iter()
        .fold(First.to_lowercase(), |Acc, Marker| Acc.split(Marker).next().unwrap_or("").to_string());
    Normalize(&First)
}

pub fn Key(Title: &str, Artist: &str) -> String {
    format!("{}|{}", _LeadArtist(Artist), Normalize(Title))
}

const VARIANTS: [&str; 14] = [
    "live", "remix", "mix", "acoustic", "cappella", "instrumental", "karaoke",
    "cover", "sped up", "slowed", "demo", "version", "edit", "tribute",
];

fn _Rank(Title: &str, Artist: &str, CandidateTitle: &str, CandidateArtist: &str) -> Option<(u8, i64)> {
    if !_Matches(Title, Artist, CandidateTitle, CandidateArtist) {
        return None;
    }

    let Want = Title.trim().to_lowercase();
    let Got = CandidateTitle.trim().to_lowercase();
    let Tier = if Want == Got {
        3
    } else if Normalize(Title) == Normalize(CandidateTitle) {
        2
    } else {
        1
    };

    let Variants = VARIANTS.iter().filter(|V| Got.contains(*V) && !Want.contains(*V)).count() as i64;
    let Extra = (Got.chars().count() as i64 - Want.chars().count() as i64).abs();

    Some((Tier, -(Variants * 1000 + Extra)))
}

fn _Best<T>(Title: &str, Artist: &str, Candidates: impl IntoIterator<Item = (String, String, T)>) -> Option<T> {
    let mut Winner: Option<((u8, i64), T)> = None;
    for (CandidateTitle, CandidateArtist, Item) in Candidates {
        let Some(Score) = _Rank(Title, Artist, &CandidateTitle, &CandidateArtist) else { continue };
        if Winner.as_ref().is_none_or(|(Best, _)| Score > *Best) {
            Winner = Some((Score, Item));
        }
    }
    Winner.map(|(_, Item)| Item)
}

fn _Matches(Title: &str, Artist: &str, CandidateTitle: &str, CandidateArtist: &str) -> bool {
    let WantTitle = Normalize(Title);
    let GotTitle = Normalize(CandidateTitle);
    let WantArtist = _LeadArtist(Artist);
    let GotArtist = Normalize(CandidateArtist);

    if WantTitle.is_empty() || GotTitle.is_empty() || WantArtist.is_empty() || GotArtist.is_empty() {
        return false;
    }

    let TitleOk = WantTitle == GotTitle || GotTitle.starts_with(&WantTitle) || WantTitle.starts_with(&GotTitle);
    let ArtistOk = GotArtist.contains(&WantArtist) || WantArtist.contains(&_LeadArtist(CandidateArtist));

    TitleOk && ArtistOk
}

pub fn ArtKey(Url: &str) -> String {
    hex::encode(&Sha256::digest(Url.as_bytes())[..16])
}

fn _Hosted(Url: &str, Hosts: &[&str]) -> bool {
    let Ok(Parsed) = reqwest::Url::parse(Url) else { return false };
    if Parsed.scheme() != "https" || Parsed.port().is_some() || !Parsed.username().is_empty() {
        return false;
    }
    let Some(Host) = Parsed.host_str() else { return false };
    Hosts.iter().any(|Allowed| {
        if let Some(Suffix) = Allowed.strip_prefix('.') {
            Host.ends_with(&format!(".{}", Suffix))
        } else {
            Host == *Allowed
        }
    })
}

fn _AllowedArt(Url: &str) -> bool {
    _Hosted(Url, &ART_HOSTS)
}

pub fn AllowedPreview(Url: &str) -> bool {
    _Hosted(Url, &PREVIEW_HOSTS)
}

pub fn RegisterArt(Db: &Database, Url: &str) -> Option<String> {
    if !_AllowedArt(Url) {
        return None;
    }
    let Key = ArtKey(Url);
    Db.CachePut("art-url", &Key, Some(Url), None, ART_STORE_TTL);
    Some(Key)
}

fn _KeepArt(Key: &str, Bytes: &[u8], Mime: &str) {
    let mut R = _Registry().lock().unwrap();
    while R.ArtUsed + Bytes.len() > ART_BUDGET && !R.ArtBytes.is_empty() {
        let Oldest = R.ArtBytes.iter().min_by_key(|(_, A)| A.At).map(|(K, _)| K.clone());
        if let Some(K) = Oldest {
            if let Some(Gone) = R.ArtBytes.remove(&K) {
                R.ArtUsed -= Gone.Bytes.len();
            }
        }
    }
    R.ArtUsed += Bytes.len();
    if let Some(Old) = R.ArtBytes.insert(Key.to_string(), Art { Bytes: Bytes.to_vec(), Mime: Mime.to_string(), At: Instant::now() }) {
        R.ArtUsed -= Old.Bytes.len();
    }
}

pub async fn Artwork(Db: &Database, Key: &str, Fallback: Option<String>) -> Option<(Vec<u8>, String)> {
    if Key.len() != 32 || !Key.bytes().all(|B| B.is_ascii_hexdigit()) {
        return None;
    }

    {
        let R = _Registry().lock().unwrap();
        if let Some(Held) = R.ArtBytes.get(Key) {
            if Held.At.elapsed() < ART_TTL {
                return Some((Held.Bytes.clone(), Held.Mime.clone()));
            }
        }
    }

    if let Some(Stored) = Db.CacheGet("art", Key) {
        if let (Some(Mime), Some(Bytes)) = (Stored.Value, Stored.Bytes) {
            _KeepArt(Key, &Bytes, &Mime);
            return Some((Bytes, Mime));
        }
    }

    let Known = Db.CacheGet("art-url", Key).and_then(|C| C.Value);
    let Url = Known.or(Fallback).filter(|U| _AllowedArt(U) && ArtKey(U) == Key)?;
    let Bytes = cover::Fetch(_ArtClient(), &Url).await?;
    let (Bytes, Mime) = cover::Accept(Bytes)?;

    Db.CachePut("art", Key, Some(&Mime), Some(&Bytes), ART_STORE_TTL);
    _KeepArt(Key, &Bytes, &Mime);
    Some((Bytes, Mime))
}

fn _TrackKey(Provider: Provider, Id: &str) -> String {
    format!("{}:{}", Provider.AsText(), Id)
}

fn _Remember(Db: &Database, List: &[Track]) {
    for T in List {
        let Ok(Encoded) = serde_json::to_string(T) else { continue };
        let Ttl = if T.Cover.is_some() { TRACK_TTL } else { TRACK_PARTIAL_TTL };
        Db.CachePut("track", &_TrackKey(T.Provider, &T.Id), Some(&Encoded), None, Ttl);
    }
}

fn _Recall(Db: &Database, Provider: Provider, Id: &str) -> Option<Track> {
    let Stored = Db.CacheGet("track", &_TrackKey(Provider, Id))?.Value?;
    serde_json::from_str(&Stored).ok()
}

async fn _Json(Request: reqwest::RequestBuilder) -> Result<serde_json::Value, String> {
    let Resp = Request.send().await.map_err(|E| E.to_string())?;
    let Status = Resp.status();
    let Text = Resp.text().await.map_err(|E| E.to_string())?;
    if !Status.is_success() {
        return Err(format!("HTTP {}", Status));
    }
    serde_json::from_str(&Text).map_err(|E| E.to_string())
}

fn _Text(Value: &serde_json::Value) -> Option<String> {
    Value.as_str().map(str::trim).filter(|S| !S.is_empty()).map(str::to_string)
}

async fn _SpotifyToken(Db: &Database) -> Option<String> {
    let ClientId = Db.GetSpotifyClientId();
    let ClientSecret = Db.GetSpotifyClientSecret();
    if ClientId.is_empty() || ClientSecret.is_empty() {
        return None;
    }

    {
        let Held = _Token().lock().unwrap();
        if let Some(T) = Held.as_ref() {
            if T.ForClient == ClientId && Instant::now() < T.ExpiresAt {
                return Some(T.Value.clone());
            }
        }
    }

    match crate::spotify::AppToken(_Client(), &ClientId, &ClientSecret).await {
        Ok((Value, Lifetime)) => {
            *_Token().lock().unwrap() = Some(SpotifyToken {
                Value: Value.clone(),
                ExpiresAt: Instant::now() + Lifetime.saturating_sub(Duration::from_secs(60)),
                ForClient: ClientId,
            });
            Some(Value)
        }
        Err(E) => {
            log!(Metadata, Warn, "spotify app token failed: {}", E);
            None
        }
    }
}

fn _SpotifyTrack(Item: &serde_json::Value) -> Option<Track> {
    let Id = _Text(&Item["id"])?;
    if Id.len() != 22 || !Id.bytes().all(|B| B.is_ascii_alphanumeric()) {
        return None;
    }
    let Title = _Text(&Item["name"])?;
    let Artist = Item["artists"]
        .as_array()?
        .iter()
        .filter_map(|A| _Text(&A["name"]))
        .collect::<Vec<String>>()
        .join(", ");
    if Artist.is_empty() {
        return None;
    }

    let Cover = _Closest(&Item["album"]["images"]);

    Some(Track {
        Provider: Provider::Spotify,
        Link: Some(format!("https://open.spotify.com/track/{}", Id)),
        Id,
        Title,
        Artist,
        Album: _Text(&Item["album"]["name"]),
        Cover,
        Explicit: Item["explicit"].as_bool(),
        DurationMs: Item["duration_ms"].as_u64().map(|D| D as u32),
    })
}

fn _Closest(Images: &serde_json::Value) -> Option<String> {
    Images.as_array()?
        .iter()
        .filter(|I| I["url"].is_string())
        .min_by_key(|I| (I["width"].as_i64().unwrap_or(0) - 640).abs())
        .and_then(|I| _Text(&I["url"]))
}

async fn _SpotifySearch(Db: &Database, Query: &str) -> Result<Vec<Track>, String> {
    let Token = _SpotifyToken(Db).await.ok_or("spotify not configured")?;
    _SpotifySearchWith(&Token, Query).await
}

async fn _SpotifySearchWith(Token: &str, Query: &str) -> Result<Vec<Track>, String> {
    let Body = _Json(
        _Client()
            .get(format!("{}/search", SPOTIFY_API))
            .bearer_auth(Token)
            .query(&[("q", Query), ("type", "track"), ("market", MARKET), ("limit", &RESULT_LIMIT.to_string())]),
    ).await?;

    Ok(Body["tracks"]["items"]
        .as_array()
        .map(|Items| Items.iter().filter_map(_SpotifyTrack).collect())
        .unwrap_or_default())
}

async fn _SpotifyLookup(Db: &Database, Id: &str) -> Option<Track> {
    if Id.len() != 22 || !Id.bytes().all(|B| B.is_ascii_alphanumeric()) {
        return None;
    }
    let Token = _SpotifyToken(Db).await?;
    let Body = _Json(
        _Client()
            .get(format!("{}/tracks/{}", SPOTIFY_API, Id))
            .bearer_auth(Token)
            .query(&[("market", MARKET)]),
    ).await.ok()?;
    _SpotifyTrack(&Body)
}

fn _LastfmImage(Images: &serde_json::Value) -> Option<String> {
    let List = Images.as_array()?;
    for Size in ["mega", "extralarge", "large", "medium"] {
        let Found = List
            .iter()
            .find(|I| I["size"].as_str() == Some(Size))
            .and_then(|I| _Text(&I["#text"]))
            .filter(|U| !U.contains(LASTFM_PLACEHOLDER));
        if Found.is_some() {
            return Found;
        }
    }
    None
}

pub fn IsLastfmPlaceholder(Url: &str) -> bool {
    Url.contains(LASTFM_PLACEHOLDER)
}

fn _LastfmId(Artist: &str, Title: &str) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(format!("{}\u{1f}{}", Artist, Title))
}

fn _LastfmUnpack(Id: &str) -> Option<(String, String)> {
    if Id.len() > 600 {
        return None;
    }
    let Raw = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(Id).ok()?;
    let Text = String::from_utf8(Raw).ok()?;
    let (Artist, Title) = Text.split_once('\u{1f}')?;
    let (Artist, Title) = (Artist.trim(), Title.trim());
    if Artist.is_empty() || Title.is_empty() || Artist.len() > 200 || Title.len() > 200 {
        return None;
    }
    Some((Artist.to_string(), Title.to_string()))
}

async fn _LastfmSearch(Db: &Database, Query: &str) -> Result<Vec<Track>, String> {
    let ApiKey = Db.GetLastfmApiKey();
    if ApiKey.is_empty() {
        return Err("lastfm not configured".into());
    }
    let Body = _Json(_Client().get(LASTFM_API).query(&[
        ("method", "track.search"),
        ("track", Query),
        ("api_key", ApiKey.as_str()),
        ("format", "json"),
        ("limit", &RESULT_LIMIT.to_string()),
    ])).await?;

    if let Some(Code) = Body["error"].as_u64() {
        return Err(format!("lastfm error {}", Code));
    }

    Ok(Body["results"]["trackmatches"]["track"]
        .as_array()
        .map(|Items| {
            Items.iter().filter_map(|Item| {
                let Title = _Text(&Item["name"])?;
                let Artist = _Text(&Item["artist"])?;
                Some(Track {
                    Provider: Provider::Lastfm,
                    Id: _LastfmId(&Artist, &Title),
                    Link: _Text(&Item["url"]),
                    Title,
                    Artist,
                    Album: None,
                    Cover: _LastfmImage(&Item["image"]),
                    Explicit: None,
                    DurationMs: None,
                })
            }).collect()
        })
        .unwrap_or_default())
}

async fn _LastfmInfo(ApiKey: &str, Artist: &str, Title: &str) -> Option<(serde_json::Value, Found)> {
    if ApiKey.is_empty() {
        return None;
    }
    let Body = _Json(_Client().get(LASTFM_API).query(&[
        ("method", "track.getInfo"),
        ("artist", Artist),
        ("track", Title),
        ("autocorrect", "1"),
        ("api_key", ApiKey),
        ("format", "json"),
    ])).await.ok()?;

    if Body["error"].is_u64() {
        return None;
    }

    let Detail = Body["track"].clone();
    let Found = Found {
        Cover: _LastfmImage(&Detail["album"]["image"]),
        Explicit: None,
        Album: _Text(&Detail["album"]["title"]),
        DurationMs: Detail["duration"]
            .as_str()
            .and_then(|D| D.parse::<u32>().ok())
            .or_else(|| Detail["duration"].as_u64().map(|D| D as u32))
            .filter(|D| *D > 0),
    };
    Some((Detail, Found))
}

async fn _LastfmLookup(Db: &Database, Id: &str) -> Option<Track> {
    let (Artist, Title) = _LastfmUnpack(Id)?;
    let (Detail, Found) = _LastfmInfo(&Db.GetLastfmApiKey(), &Artist, &Title).await?;
    let Title = _Text(&Detail["name"])?;
    let Artist = _Text(&Detail["artist"]["name"])?;
    Some(Track {
        Provider: Provider::Lastfm,
        Id: Id.to_string(),
        Link: _Text(&Detail["url"]),
        Title,
        Artist,
        Album: Found.Album,
        Cover: Found.Cover,
        Explicit: None,
        DurationMs: Found.DurationMs,
    })
}

fn _DeezerTrack(Item: &serde_json::Value) -> Option<Track> {
    let Id = Item["id"].as_u64()?.to_string();
    let Title = _Text(&Item["title"])?;
    let Artist = _Text(&Item["artist"]["name"])?;
    let Cover = _Text(&Item["album"]["cover_xl"]).or_else(|| _Text(&Item["album"]["cover_big"]));
    Some(Track {
        Provider: Provider::Deezer,
        Link: _Text(&Item["link"]).or_else(|| Some(format!("https://www.deezer.com/track/{}", Id))),
        Id,
        Title,
        Artist,
        Album: _Text(&Item["album"]["title"]),
        Cover,
        Explicit: Item["explicit_lyrics"].as_bool(),
        DurationMs: Item["duration"].as_u64().map(|D| (D * 1000) as u32).filter(|D| *D > 0),
    })
}

async fn _DeezerSearch(Query: &str) -> Result<Vec<Track>, String> {
    let Body = _Json(_Client()
        .get(format!("{}/search", DEEZER_API))
        .query(&[("q", Query), ("limit", &RESULT_LIMIT.to_string())])).await?;
    if Body["error"].is_object() {
        return Err(format!("deezer error {}", Body["error"]["code"]));
    }
    Ok(Body["data"]
        .as_array()
        .map(|Items| Items.iter().filter_map(_DeezerTrack).collect())
        .unwrap_or_default())
}

async fn _DeezerLookup(Id: &str) -> Option<Track> {
    if Id.is_empty() || Id.len() > 20 || !Id.bytes().all(|B| B.is_ascii_digit()) {
        return None;
    }
    let Body = _Json(_Client().get(format!("{}/track/{}", DEEZER_API, Id))).await.ok()?;
    if Body["error"].is_object() {
        return None;
    }
    _DeezerTrack(&Body)
}

async fn _DeezerMatch(Title: &str, Artist: &str) -> Option<Found> {
    let Strict = format!("artist:\"{}\" track:\"{}\"", _Quote(&_LeadArtistRaw(Artist)), _Quote(Title));
    let mut Candidates = _DeezerSearch(&Strict).await.unwrap_or_default();
    if Candidates.is_empty() {
        Candidates = _DeezerSearch(&format!("{} {}", Artist, Title)).await.unwrap_or_default();
    }
    let Hit = _Best(Title, Artist, Candidates.into_iter().map(|C| (C.Title.clone(), C.Artist.clone(), C)))?;
    Some(Hit.into())
}

fn _Quote(Text: &str) -> String {
    Text.replace('"', "")
}

fn _LeadArtistRaw(Artist: &str) -> String {
    Artist.split([',', '&', ';']).next().unwrap_or(Artist).trim().to_string()
}

async fn _ItunesMatch(Title: &str, Artist: &str) -> Option<Found> {
    let Term = format!("{} {}", _LeadArtistRaw(Artist), Title);
    let Body = _Json(_Client().get(ITUNES_API).query(&[
        ("term", Term.as_str()),
        ("media", "music"),
        ("entity", "song"),
        ("country", MARKET),
        ("limit", "8"),
    ])).await.ok()?;

    let Hit = _Best(Title, Artist, Body["results"].as_array()?.iter().map(|R| (
        R["trackName"].as_str().unwrap_or("").to_string(),
        R["artistName"].as_str().unwrap_or("").to_string(),
        R,
    )))?;

    Some(Found {
        Cover: _Text(&Hit["artworkUrl100"]).map(|U| U.replace("100x100bb", "600x600bb")),
        Explicit: match Hit["trackExplicitness"].as_str() {
            Some("explicit") => Some(true),
            Some("notExplicit") | Some("cleaned") => Some(false),
            _ => None,
        },
        Album: _Text(&Hit["collectionName"]),
        DurationMs: Hit["trackTimeMillis"].as_u64().map(|D| D as u32),
    })
}

async fn _Fill(LastfmKey: &str, mut T: Track) -> Track {
    if T.Complete() {
        return T;
    }

    let NeedsLastfm = T.Cover.is_none() || T.Album.is_none() || T.DurationMs.is_none();
    let (Lastfm, Deezer, Itunes) = tokio::join!(
        async { if NeedsLastfm { _LastfmInfo(LastfmKey, &T.Artist, &T.Title).await.map(|(_, F)| F) } else { None } },
        _DeezerMatch(&T.Title, &T.Artist),
        _ItunesMatch(&T.Title, &T.Artist),
    );

    for Found in [Deezer, Itunes, Lastfm].into_iter().flatten() {
        T.Absorb(Found);
    }

    if T.Cover.is_none() && T.Provider != Provider::Spotify {
        if let Some(Hit) = _SpotifyWebMatch(&T.Title, &T.Artist).await {
            T.Absorb(Hit.into());
        }
    }

    T
}

async fn _Enrich(Db: &Database, mut List: Vec<Track>, Query: Option<&str>) -> Vec<Track> {
    if let Some(Query) = Query.filter(|_| List.iter().any(|T| T.Cover.is_none())) {
        match _SpotifyWebSearch(Query).await {
            Ok(Pool) => {
                for T in List.iter_mut().filter(|T| T.Cover.is_none()) {
                    let Hit = _Best(&T.Title, &T.Artist, Pool.iter().map(|C| (C.Title.clone(), C.Artist.clone(), C)));
                    if let Some(Hit) = Hit {
                        T.Absorb(Hit.clone().into());
                    }
                }
            }
            Err(E) => log!(Metadata, Warn, "spotify web search failed: {}", E),
        }
    }

    let LastfmKey = Db.GetLastfmApiKey();
    let Work = List.into_iter().map(|T| {
        let Key = LastfmKey.clone();
        async move {
            let Original = T.clone();
            match tokio::time::timeout(ENRICH_BUDGET, _Fill(&Key, T)).await {
                Ok(Done) => Done,
                Err(_) => {
                    log!(Metadata, Warn, "enrichment timed out for {} - {}", Original.Artist, Original.Title);
                    Original
                }
            }
        }
    });

    join_all(Work).await
}

pub fn ActiveSource(Db: &Database) -> Provider {
    if !Db.GetSpotifyClientId().is_empty() && !Db.GetSpotifyClientSecret().is_empty() {
        Provider::Spotify
    } else if !Db.GetLastfmApiKey().is_empty() {
        Provider::Lastfm
    } else {
        Provider::Deezer
    }
}

pub async fn Search(Db: &Database, Query: &str) -> Result<(Provider, Vec<Track>), String> {
    let CacheKey = format!("{}|{}", ActiveSource(Db).AsText(), Query.to_lowercase().split_whitespace().collect::<Vec<&str>>().join(" "));
    if let Some(Stored) = Db.CacheGet("search", &CacheKey).and_then(|C| C.Value) {
        if let Ok(Known) = serde_json::from_str::<Answer>(&Stored) {
            return Ok((Known.Source, Known.Tracks));
        }
    }

    let Result = _SearchUncached(Db, Query).await;

    if let Ok((Source, Tracks)) = &Result {
        if !Tracks.is_empty() {
            let Complete = Tracks.iter().all(|T| T.Cover.is_some());
            let Answer = Answer { Source: *Source, Tracks: Tracks.clone() };
            if let Ok(Encoded) = serde_json::to_string(&Answer) {
                Db.CachePut("search", &CacheKey, Some(&Encoded), None, if Complete { QUERY_TTL } else { QUERY_PARTIAL_TTL });
            }
        }
    }

    Result
}

async fn _SearchUncached(Db: &Database, Query: &str) -> Result<(Provider, Vec<Track>), String> {
    let mut Failures: Vec<String> = Vec::new();

    let Chain = [Provider::Spotify, Provider::Lastfm, Provider::Deezer];
    for Source in Chain {
        let Attempt = match Source {
            Provider::Spotify if !Db.GetSpotifyClientId().is_empty() => _SpotifySearch(Db, Query).await,
            Provider::Lastfm if !Db.GetLastfmApiKey().is_empty() => _LastfmSearch(Db, Query).await,
            Provider::Deezer => _DeezerSearch(Query).await,
            _ => continue,
        };

        match Attempt {
            Ok(Found) if !Found.is_empty() => {
                let Pooled = (Source != Provider::Spotify).then_some(Query);
                let Filled = _Enrich(Db, Found, Pooled).await;
                _Remember(Db, &Filled);
                return Ok((Source, Filled));
            }
            Ok(_) => Failures.push(format!("{}: empty", Source.AsText())),
            Err(E) => {
                log!(Metadata, Warn, "{} search failed: {}", Source.AsText(), E);
                Failures.push(format!("{}: {}", Source.AsText(), E));
            }
        }
    }

    if Failures.iter().all(|F| F.ends_with(": empty")) {
        return Ok((ActiveSource(Db), Vec::new()));
    }

    Err(Failures.join("; "))
}

pub async fn Resolve(Db: &Database, Provider: Provider, Id: &str) -> Option<Track> {
    if let Some(Known) = _Recall(Db, Provider, Id) {
        return Some(Known);
    }

    let Fetched = match Provider {
        Provider::Spotify => _SpotifyLookup(Db, Id).await,
        Provider::Lastfm => _LastfmLookup(Db, Id).await,
        Provider::Deezer => _DeezerLookup(Id).await,
    }?;

    let Filled = _Enrich(Db, vec![Fetched], None).await;
    _Remember(Db, &Filled);
    Filled.into_iter().next()
}

pub async fn FindCoverUrl(Db: &Database, Title: &str, Artist: &str) -> Option<String> {
    let LastfmKey = Db.GetLastfmApiKey();
    if let Some((_, Found)) = _LastfmInfo(&LastfmKey, Artist, Title).await {
        if Found.Cover.is_some() {
            return Found.Cover;
        }
    }
    if let Some(Found) = _DeezerMatch(Title, Artist).await {
        if Found.Cover.is_some() {
            return Found.Cover;
        }
    }
    if let Some(Cover) = _ItunesMatch(Title, Artist).await.and_then(|F| F.Cover) {
        return Some(Cover);
    }
    _SpotifyWebMatch(Title, Artist).await.and_then(|T| T.Cover)
}

pub async fn DeezerCoverUrl(Title: &str, Artist: &str) -> Option<String> {
    _DeezerMatch(Title, Artist).await.and_then(|F| F.Cover)
}

async fn _Embed(Id: &str) -> Option<serde_json::Value> {
    if Id.is_empty() || Id.len() > 32 || !Id.bytes().all(|B| B.is_ascii_alphanumeric()) {
        return None;
    }
    let Resp = _Client().get(format!("{}/{}", SPOTIFY_EMBED, Id)).send().await.ok()?;
    if !Resp.status().is_success() {
        return None;
    }
    let Html = Resp.text().await.ok()?;
    let Start = Html.find("<script id=\"__NEXT_DATA__\"")?;
    let Open = Start + Html[Start..].find('>')? + 1;
    let Close = Open + Html[Open..].find("</script>")?;
    let Data: serde_json::Value = serde_json::from_str(&Html[Open..Close]).ok()?;
    _Keep(&Data["props"]["pageProps"]["state"]["settings"]["session"]);
    Some(Data)
}

fn _Keep(Session: &serde_json::Value) {
    let Some(Value) = _Text(&Session["accessToken"]) else { return };
    if Session["isAnonymous"].as_bool() != Some(true) {
        return;
    }
    let Now = SystemTime::now().duration_since(UNIX_EPOCH).map(|D| D.as_millis() as u64).unwrap_or(0);
    let Left = Session["accessTokenExpirationTimestampMs"].as_u64().unwrap_or(0).saturating_sub(Now);
    if Left < 120_000 {
        return;
    }
    let ExpiresAt = Instant::now() + Duration::from_millis(Left - 60_000);
    *_Anon().lock().unwrap() = Some((Value, ExpiresAt));
}

async fn _AnonToken() -> Option<String> {
    let Held = _Anon().lock().unwrap().clone();
    if let Some((Value, ExpiresAt)) = Held {
        if Instant::now() < ExpiresAt {
            return Some(Value);
        }
    }
    _Embed(SPOTIFY_TOKEN_TRACK).await?;
    _Anon().lock().unwrap().as_ref().map(|(Value, _)| Value.clone())
}

async fn _SpotifyPreview(Id: &str) -> Option<String> {
    let Data = _Embed(Id).await?;
    _Text(&Data["props"]["pageProps"]["state"]["data"]["entity"]["audioPreview"]["url"])
}

async fn _SpotifyMatch(Db: &Database, Title: &str, Artist: &str) -> Option<String> {
    let Some(Token) = _SpotifyToken(Db).await else {
        return _SpotifyWebMatch(Title, Artist).await.map(|T| T.Id);
    };
    let Query = format!("track:\"{}\" artist:\"{}\"", _Quote(Title), _Quote(&_LeadArtistRaw(Artist)));
    let Candidates = _SpotifySearchWith(&Token, &Query).await.ok()?;
    _Best(Title, Artist, Candidates.into_iter().map(|C| (C.Title.clone(), C.Artist.clone(), C.Id)))
}

fn _PathfinderTrack(Data: &serde_json::Value) -> Option<Track> {
    let Id = _Text(&Data["id"])?;
    if Id.len() != 22 || !Id.bytes().all(|B| B.is_ascii_alphanumeric()) {
        return None;
    }
    let Title = _Text(&Data["name"])?;
    let Artist = Data["artists"]["items"]
        .as_array()?
        .iter()
        .filter_map(|A| _Text(&A["profile"]["name"]))
        .collect::<Vec<String>>()
        .join(", ");
    if Artist.is_empty() {
        return None;
    }
    let Album = &Data["albumOfTrack"];

    Some(Track {
        Provider: Provider::Spotify,
        Link: Some(format!("https://open.spotify.com/track/{}", Id)),
        Id,
        Title,
        Artist,
        Album: _Text(&Album["name"]),
        Cover: _Closest(&Album["coverArt"]["sources"]),
        Explicit: Data["contentRating"]["label"].as_str().map(|L| L == "EXPLICIT"),
        DurationMs: Data["duration"]["totalMilliseconds"].as_u64().map(|D| D as u32).filter(|D| *D > 0),
    })
}

async fn _SpotifyWebSearch(Term: &str) -> Result<Vec<Track>, String> {
    let Token = _AnonToken().await.ok_or("anonymous spotify token unavailable")?;
    let Payload = serde_json::json!({
        "variables": {
            "searchTerm": Term,
            "offset": 0,
            "limit": 20,
            "numberOfTopResults": 5,
            "includeAudiobooks": false,
            "includeArtistHasConcertsField": false,
            "includePreReleases": false,
            "includeLocalConcertsField": false,
            "includeAuthors": false,
        },
        "operationName": "searchDesktop",
        "extensions": { "persistedQuery": { "version": 1, "sha256Hash": SPOTIFY_SEARCH_HASH } },
    });
    let Body = _Json(
        _Client()
            .post(SPOTIFY_PATHFINDER)
            .bearer_auth(&Token)
            .header(reqwest::header::USER_AGENT, BROWSER_AGENT)
            .header(reqwest::header::CONTENT_TYPE, "application/json;charset=UTF-8")
            .header("app-platform", "WebPlayer")
            .body(Payload.to_string()),
    ).await.inspect_err(|_| *_Anon().lock().unwrap() = None)?;

    let Items = Body["data"]["searchV2"]["tracksV2"]["items"]
        .as_array()
        .ok_or_else(|| format!("unexpected pathfinder reply: {}", Body["errors"][0]["message"].as_str().unwrap_or("no data")))?;
    Ok(Items.iter().filter_map(|I| _PathfinderTrack(&I["item"]["data"])).collect())
}

async fn _SpotifyWebMatch(Title: &str, Artist: &str) -> Option<Track> {
    let Term = format!("{} {}", _LeadArtistRaw(Artist), Title);
    let Candidates = match _SpotifyWebSearch(&Term).await {
        Ok(Found) => Found,
        Err(E) => {
            log!(Metadata, Warn, "spotify web search failed: {}", E);
            return None;
        }
    };
    _Best(Title, Artist, Candidates.into_iter().map(|C| (C.Title.clone(), C.Artist.clone(), C)))
}

pub async fn PreviewUrl(Db: &Database, Provider: Provider, Id: &str, Title: &str, Artist: &str) -> Option<String> {
    let CacheKey = _TrackKey(Provider, Id);
    if let Some(Url) = Db.CacheGet("preview", &CacheKey).and_then(|Stored| Stored.Value).filter(|U| AllowedPreview(U)) {
        return Some(Url);
    }

    let SpotifyId = match Provider {
        Provider::Spotify => Some(Id.to_string()),
        _ => _SpotifyMatch(Db, Title, Artist).await,
    };
    let Found = match SpotifyId {
        Some(SpotifyId) => _SpotifyPreview(&SpotifyId).await.filter(|U| AllowedPreview(U)),
        None => None,
    };

    if let Some(Url) = Found.as_deref() {
        Db.CachePut("preview", &CacheKey, Some(Url), None, PREVIEW_TTL);
    }
    Found
}

pub async fn Download(Url: &str) -> Option<(Vec<u8>, String)> {
    cover::Fetch(_Client(), Url).await.and_then(cover::Accept)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_decorations() {
        assert_eq!(Normalize("Never Gonna Give You Up (Remastered 2022)"), "never gonna give you up");
        assert_eq!(Normalize("Song - Radio Edit"), "song");
        assert_eq!(Normalize("Tak Tak feat. Kto Ś"), "tak tak");
        assert_eq!(Normalize("Żółć [Live]"), "żółć");
    }

    #[test]
    fn keys_ignore_featured_artists() {
        assert_eq!(Key("Bad Habits", "Ed Sheeran"), Key("Bad Habits (Acoustic)", "Ed Sheeran, Someone"));
        assert_ne!(Key("Bad Habits", "Ed Sheeran"), Key("Shivers", "Ed Sheeran"));
    }

    #[test]
    fn matches_across_providers() {
        assert!(_Matches("Blinding Lights", "The Weeknd", "Blinding Lights", "The Weeknd"));
        assert!(_Matches("Blinding Lights", "The Weeknd, Someone", "Blinding Lights - Remix", "The Weeknd"));
        assert!(!_Matches("Blinding Lights", "The Weeknd", "Save Your Tears", "The Weeknd"));
        assert!(!_Matches("Blinding Lights", "The Weeknd", "Blinding Lights", "Cover Band"));
    }

    #[test]
    fn prefers_plain_versions_over_decorated_ones() {
        let Candidates = vec![
            ("Bohemian Rhapsody (Operatic Section / 2011 A Cappella Mix)".to_string(), "Queen".to_string(), "mix"),
            ("Bohemian Rhapsody - Remastered 2011".to_string(), "Queen".to_string(), "remaster"),
            ("Bohemian Rhapsody".to_string(), "Queen".to_string(), "plain"),
            ("Bohemian Rhapsody".to_string(), "Cover Band".to_string(), "impostor"),
        ];
        assert_eq!(_Best("Bohemian Rhapsody", "Queen", Candidates.clone()), Some("plain"));
        assert_eq!(_Best("Bohemian Rhapsody - Remastered 2011", "Queen", Candidates.clone()), Some("remaster"));
        assert_eq!(_Best("Bohemian Rhapsody", "Queen", Candidates[..2].to_vec()), Some("remaster"));
        assert_eq!(_Best("Radio Ga Ga", "Queen", Candidates), None);
    }

    #[test]
    fn art_allowlist_blocks_foreign_hosts() {
        assert!(_AllowedArt("https://i.scdn.co/image/ab67616d"));
        assert!(_AllowedArt("https://e-cdns-images.dzcdn.net/images/cover/x/1000x1000.jpg"));
        assert!(_AllowedArt("https://is1-ssl.mzstatic.com/image/thumb/a.jpg"));
        assert!(!_AllowedArt("http://i.scdn.co/image/x"));
        assert!(!_AllowedArt("https://evil.com/i.scdn.co"));
        assert!(!_AllowedArt("https://i.scdn.co.evil.com/x"));
        assert!(!_AllowedArt("https://127.0.0.1/x"));
        assert!(!_AllowedArt("https://i.scdn.co:8443/x"));
        assert!(!_AllowedArt("https://user@i.scdn.co/x"));
    }

    #[test]
    fn lastfm_ids_roundtrip_and_reject_garbage() {
        let Id = _LastfmId("Dawid Podsiadło", "Małomiasteczkowy");
        assert_eq!(_LastfmUnpack(&Id), Some(("Dawid Podsiadło".into(), "Małomiasteczkowy".into())));
        assert_eq!(_LastfmUnpack("!!!"), None);
        assert_eq!(_LastfmUnpack(&base64::engine::general_purpose::URL_SAFE_NO_PAD.encode("no separator")), None);
    }
}
