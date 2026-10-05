#![allow(non_snake_case)]

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use axum::body::Bytes;

use crate::db::Database;
use crate::metadata;
use crate::voting;
use crate::log::{Subsystem::Metadata, Level::*};

const CLIP_TTL: Duration = Duration::from_secs(12 * 60 * 60);
const CLIP_BUDGET: usize = 64 * 1024 * 1024;
const CLIP_MAX: usize = 3 * 1024 * 1024;
const CLIP_STORE_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);

#[derive(Clone)]
pub struct Clip {
    pub Bytes: Bytes,
    pub Mime: String,
}

struct Held {
    Clip: Clip,
    At: Instant,
}

#[derive(Default)]
struct Store {
    Clips: HashMap<i64, Held>,
    Used: usize,
}

fn _Store() -> &'static Mutex<Store> {
    static S: OnceLock<Mutex<Store>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(Store::default()))
}

fn _Client() -> Option<&'static reqwest::Client> {
    static C: OnceLock<Option<reqwest::Client>> = OnceLock::new();
    C.get_or_init(|| {
        match reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(4))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(concat!("zstio-tv/", env!("CARGO_PKG_VERSION")))
            .build()
        {
            Ok(Client) => Some(Client),
            Err(E) => {
                fail!(Metadata, E, "could not build preview client");
                None
            }
        }
    }).as_ref()
}

fn _Mime(Raw: &str) -> Option<String> {
    let Mime = Raw.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
    Mime.starts_with("audio/").then_some(Mime)
}

async fn _Fetch(Url: &str) -> Option<Clip> {
    if !metadata::AllowedPreview(Url) {
        return None;
    }

    let Resp = _Client()?.get(Url).send().await.ok()?;
    if !Resp.status().is_success() {
        return None;
    }

    let Mime = Resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|V| V.to_str().ok())
        .and_then(_Mime)?;

    if Resp.content_length().is_some_and(|Length| Length as usize > CLIP_MAX) {
        return None;
    }

    let Bytes = Resp.bytes().await.ok()?;
    if Bytes.is_empty() || Bytes.len() > CLIP_MAX {
        return None;
    }

    Some(Clip { Bytes, Mime })
}

fn _Keep(Song: i64, Clip: &Clip) {
    let mut S = _Store().lock().unwrap();
    while S.Used + Clip.Bytes.len() > CLIP_BUDGET && !S.Clips.is_empty() {
        let Oldest = S.Clips.iter().min_by_key(|(_, H)| H.At).map(|(K, _)| *K);
        if let Some(Key) = Oldest {
            if let Some(Gone) = S.Clips.remove(&Key) {
                S.Used -= Gone.Clip.Bytes.len();
            }
        }
    }
    S.Used += Clip.Bytes.len();
    if let Some(Old) = S.Clips.insert(Song, Held { Clip: Clip.clone(), At: Instant::now() }) {
        S.Used -= Old.Clip.Bytes.len();
    }
}

pub async fn Load(Db: &Database, Song: i64, Approved: bool) -> Option<Clip> {
    let Source = voting::PreviewSource(Db, Song, Approved)?;

    {
        let S = _Store().lock().unwrap();
        if let Some(Held) = S.Clips.get(&Song).filter(|H| H.At.elapsed() < CLIP_TTL) {
            return Some(Held.Clip.clone());
        }
    }

    let CacheKey = Song.to_string();
    if let Some((Mime, Bytes)) = Db.CacheGet("clip", &CacheKey).and_then(|Stored| Stored.Value.zip(Stored.Bytes)) {
        let Clip = Clip { Bytes: Bytes.into(), Mime };
        _Keep(Song, &Clip);
        return Some(Clip);
    }

    let (Provider, ExtId, Title, Artist) = Source;
    let Found = match metadata::PreviewUrl(Db, Provider, &ExtId, &Title, &Artist).await {
        Some(Url) => _Fetch(&Url).await,
        None => None,
    };

    let Some(Clip) = Found else {
        log!(Metadata, Info, "no preview for {} - {}", Artist, Title);
        return None;
    };

    Db.CachePut("clip", &CacheKey, Some(&Clip.Mime), Some(&Clip.Bytes), CLIP_STORE_TTL);
    _Keep(Song, &Clip);
    Some(Clip)
}
