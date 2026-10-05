#![allow(non_snake_case)]

use std::sync::{Arc, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tauri::Emitter;

use crate::AppState;
use crate::cover;
use crate::db::Database;
use crate::log::{Subsystem::News, Level::*};

const SOURCE: &str = "https://zstiojar.edu.pl/wp-json/wp/v2/posts?per_page=1&_embed=wp:featuredmedia&_fields=id,date,title,content,featured_media,_links,_embedded";
const REFRESH: Duration = Duration::from_secs(10 * 60);
const RETRY: Duration = Duration::from_secs(60);
const KEEP: Duration = Duration::from_secs(10 * 365 * 24 * 60 * 60);
const IMAGE_LIMIT: usize = 15 * 1024 * 1024;
const KEY: &str = "latest";

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct Stored {
    Id: i64,
    Date: String,
    Title: String,
    Content: String,
    ImageUrl: Option<String>,
    ImageTag: Option<String>,
    Fetched: i64,
}

impl Stored {
    fn Same(&self, Other: &Stored) -> bool {
        self.Id == Other.Id
            && self.Date == Other.Date
            && self.Title == Other.Title
            && self.Content == Other.Content
            && self.ImageTag == Other.ImageTag
    }
}

fn _Now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|D| D.as_secs() as i64).unwrap_or(0)
}

fn _Client() -> &'static reqwest::Client {
    static C: OnceLock<reqwest::Client> = OnceLock::new();
    C.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .connect_timeout(Duration::from_secs(6))
            .user_agent(concat!("zstio-tv/", env!("CARGO_PKG_VERSION")))
            .build()
            .unwrap_or_default()
    })
}

fn _Load(Db: &Database) -> Option<Stored> {
    let Raw = Db.CacheGet("news", KEY)?.Value?;
    serde_json::from_str(&Raw).ok()
}

fn _Save(Db: &Database, Post: &Stored) {
    match serde_json::to_string(Post) {
        Ok(Raw) => Db.CachePut("news", KEY, Some(&Raw), None, KEEP),
        Err(E) => { fail!(News, E, "could not encode cached article"); }
    }
}

pub fn Current(Db: &Database) -> Value {
    match _Load(Db) {
        Some(Post) => json!({
            "Id": Post.Id,
            "Date": Post.Date,
            "Title": Post.Title,
            "Content": Post.Content,
            "Image": Post.ImageTag.map(|Tag| format!("http://127.0.0.1:9999/api/news/image?v={}", Tag)),
        }),
        None => Value::Null,
    }
}

pub fn Image(Db: &Database) -> Option<(Vec<u8>, String)> {
    let Stored = Db.CacheGet("news-image", KEY)?;
    Stored.Bytes.zip(Stored.Value)
}

async fn _Image(Url: &str) -> Result<(Vec<u8>, &'static str), String> {
    let Resp = _Client().get(Url).send().await.map_err(|E| E.to_string())?;
    if !Resp.status().is_success() {
        return Err(format!("image HTTP {}", Resp.status()));
    }
    if Resp.content_length().is_some_and(|L| L as usize > IMAGE_LIMIT) {
        return Err("image too large".into());
    }
    let Bytes = Resp.bytes().await.map_err(|E| E.to_string())?;
    if Bytes.len() > IMAGE_LIMIT {
        return Err("image too large".into());
    }
    let Mime = cover::SniffMime(&Bytes).ok_or("image format not recognised")?;
    Ok((Bytes.to_vec(), Mime))
}

async fn _Fetch(Db: &Database) -> Result<bool, String> {
    let Resp = _Client().get(SOURCE).send().await.map_err(|E| E.to_string())?;
    if !Resp.status().is_success() {
        return Err(format!("HTTP {}", Resp.status()));
    }
    let Text = Resp.text().await.map_err(|E| E.to_string())?;
    let Body: Value = serde_json::from_str(&Text).map_err(|E| E.to_string())?;
    let Raw = Body.get(0).ok_or("no posts returned")?;

    let Title = Raw["title"]["rendered"].as_str().unwrap_or("").trim().to_string();
    if Title.is_empty() {
        return Err("post has no title".into());
    }

    let Previous = _Load(Db);
    let ImageUrl = Raw["_embedded"]["wp:featuredmedia"][0]["source_url"].as_str().map(str::to_string);

    let Kept = Previous.as_ref().filter(|P| P.ImageUrl == ImageUrl && P.ImageTag.is_some() && Image(Db).is_some());
    let ImageTag = match (&ImageUrl, Kept) {
        (_, Some(P)) => P.ImageTag.clone(),
        (Some(Url), None) => match _Image(Url).await {
            Ok((Bytes, Mime)) => {
                let Tag = hex::encode(&Sha256::digest(&Bytes)[..8]);
                Db.CachePut("news-image", KEY, Some(Mime), Some(&Bytes), KEEP);
                Some(Tag)
            }
            Err(E) => {
                log!(News, Warn, "featured image failed: {}", E);
                None
            }
        },
        (None, None) => None,
    };

    let Post = Stored {
        Id: Raw["id"].as_i64().unwrap_or(0),
        Date: Raw["date"].as_str().unwrap_or("").to_string(),
        Title,
        Content: Raw["content"]["rendered"].as_str().unwrap_or("").to_string(),
        ImageUrl,
        ImageTag,
        Fetched: _Now(),
    };

    let Changed = Previous.as_ref().is_none_or(|P| !P.Same(&Post));
    _Save(Db, &Post);
    Ok(Changed)
}

pub fn Spawn(State: Arc<AppState>) {
    tokio::spawn(async move {
        loop {
            let Age = _Load(&State.Db).map(|P| _Now().saturating_sub(P.Fetched).max(0) as u64);
            let Due = Age.is_none_or(|Seconds| Seconds >= REFRESH.as_secs());

            let Wait = if !Due {
                REFRESH - Duration::from_secs(Age.unwrap_or(0))
            } else {
                match _Fetch(&State.Db).await {
                    Ok(Changed) => {
                        if Changed {
                            log!(News, Info, "article updated");
                            if let Some(Handle) = State.Handle.get() {
                                Handle.emit("news-changed", Current(&State.Db)).ok();
                            }
                        }
                        REFRESH
                    }
                    Err(E) => {
                        log!(News, Warn, "refresh failed, keeping cached article: {}", E);
                        RETRY
                    }
                }
            };

            tokio::time::sleep(Wait).await;
        }
    });
}
