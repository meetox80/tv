#![allow(non_snake_case)]

use std::sync::Arc;
use axum::{body::Bytes, extract::State, http::{header, HeaderMap, StatusCode}, Json, response::{IntoResponse, Response}};

use crate::AppState;
use crate::cover;
use crate::log::{Subsystem::Server, Level::*};
use super::PasswordRequest;

pub async fn Login(
    State(AppState): State<Arc<AppState>>,
    Json(Req): Json<PasswordRequest>,
) -> Response {
    if AppState.Db.CheckPassword(&Req.password) {
        let Token = AppState.CreateSession();
        log!(Server, Info, "panel login successful");
        (StatusCode::OK, Json(serde_json::json!({ "Token": Token }))).into_response()
    } else {
        log!(Server, Warn, "panel login failed");
        StatusCode::UNAUTHORIZED.into_response()
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct PokeRequest {
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "Artist")]
    artist: String,
    #[serde(rename = "Cover", default)]
    cover: Option<String>,
    #[serde(rename = "CoverUrl", default)]
    cover_url: Option<String>,
    #[serde(rename = "CoverLookup", default)]
    cover_lookup: Option<String>,
}

fn _KeyMatches(AppState: &Arc<AppState>, Params: &std::collections::HashMap<String, String>) -> bool {
    Params.get("key").map_or(false, |K| *K == AppState.Db.GetApiKey())
}

async fn _ResolveCover(Req: &PokeRequest) -> Option<(Vec<u8>, String)> {
    if let Some(Raw) = Req.cover.as_deref().filter(|R| !R.trim().is_empty()) {
        return cover::DecodeBase64(Raw).and_then(cover::Accept);
    }

    let Url = Req.cover_url.as_deref().filter(|U| !U.trim().is_empty())?;
    let Client = reqwest::Client::builder().build().ok()?;
    cover::Fetch(&Client, Url.trim()).await.and_then(cover::Accept)
}

pub async fn GetNewsImage(State(AppState): State<Arc<AppState>>) -> Response {
    match crate::news::Image(&AppState.Db) {
        Some((Bytes, Mime)) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, Mime),
                (header::CACHE_CONTROL, "public, max-age=31536000, immutable".to_string()),
            ],
            Bytes,
        ).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn GetCover(State(AppState): State<Arc<AppState>>) -> Response {
    let Held = AppState.CurrentCover.lock().unwrap().clone();

    match Held {
        Some(Art) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, Art.Mime),
                (header::CACHE_CONTROL, "public, max-age=31536000, immutable".to_string()),
            ],
            Art.Bytes,
        ).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn PutCover(
    State(AppState): State<Arc<AppState>>,
    axum::extract::Query(Params): axum::extract::Query<std::collections::HashMap<String, String>>,
    Headers: HeaderMap,
    Body: Bytes,
) -> StatusCode {
    if !_KeyMatches(&AppState, &Params) {
        log!(Server, Warn, "cover upload: invalid key");
        return StatusCode::UNAUTHORIZED;
    }

    if Body.len() > cover::MAX_BYTES {
        return StatusCode::PAYLOAD_TOO_LARGE;
    }

    let Declared = Headers
        .get(header::CONTENT_TYPE)
        .and_then(|V| V.to_str().ok())
        .unwrap_or("");

    let Decoded = if Declared.starts_with("text/") || Declared.is_empty() {
        String::from_utf8(Body.to_vec())
            .ok()
            .and_then(|Raw| cover::DecodeBase64(&Raw))
    } else {
        Some(Body.to_vec())
    };

    match Decoded.and_then(cover::Accept) {
        Some(Art) => {
            AppState.SetCover(Some(Art));
            log!(Server, Info, "cover updated ({} bytes)", Body.len());
            StatusCode::OK
        }
        None => StatusCode::BAD_REQUEST,
    }
}

pub async fn PlaybackPoke(
    State(AppState): State<Arc<AppState>>,
    axum::extract::Query(Params): axum::extract::Query<std::collections::HashMap<String, String>>,
    Json(Body): Json<PokeRequest>,
) -> StatusCode {
    if !_KeyMatches(&AppState, &Params) {
        log!(Server, Warn, "playback poke: invalid key");
        return StatusCode::UNAUTHORIZED;
    }

    if Body.title.is_empty() {
        return StatusCode::BAD_REQUEST;
    }

    let Lookup = match Body.cover_lookup.as_deref().map(str::trim).filter(|L| !L.is_empty()) {
        None => None,
        Some(Raw) => match crate::CoverLookup::Parse(Raw) {
            Some(Mode) => Some(Mode),
            None => return StatusCode::BAD_REQUEST,
        },
    };

    AppState.SetTrack(Some(crate::NowPlaying {
        Title: Body.title.clone(),
        Artist: Body.artist.clone(),
    }));

    if let Some(Art) = _ResolveCover(&Body).await {
        AppState.SetCover(Some(Art));
    } else if let Some(Mode) = Lookup {
        AppState.LookupCover(Body.title.clone(), Body.artist.clone(), Mode);
    }

    log!(Server, Info, "playback poke: updated");
    StatusCode::OK
}

pub async fn GetCurrentTrack(
    State(AppState): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let Held = AppState.CurrentTrack.lock().unwrap().clone();
    match Held {
        Some(T) => Json(serde_json::json!({
            "Title": T.Title,
            "Artist": T.Artist,
            "Cover": AppState.CoverPath(),
        })),
        None => Json(serde_json::json!(null)),
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct LessonPokeRequest {
    #[serde(rename = "Duration")]
    duration: u8,
}

pub async fn LessonPoke(
    State(AppState): State<Arc<AppState>>,
    axum::extract::Query(Params): axum::extract::Query<std::collections::HashMap<String, String>>,
    Json(Body): Json<LessonPokeRequest>,
) -> StatusCode {
    if !_KeyMatches(&AppState, &Params) {
        log!(Server, Warn, "lesson poke: invalid key");
        return StatusCode::UNAUTHORIZED;
    }

    if Body.duration != 30 && Body.duration != 45 {
        return StatusCode::BAD_REQUEST;
    }

    AppState.Db.SetLessonDuration(Body.duration);
    AppState.NotifyScreen();

    log!(Server, Info, "lesson poke: {} min", Body.duration);
    StatusCode::OK
}

pub async fn GetPlaybackSource(
    State(AppState): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let Method = AppState.Db.GetScrobbleMethod();
    let Fault = AppState.LastfmError.lock().unwrap().clone();

    let Ready = match Method.as_str() {
        "lastfm" => AppState.Db.IsLastfmVerified() && Fault.is_none(),
        "spotify" => AppState.Db.IsSpotifyConnected(),
        "serial" => AppState.SerialOpen.load(std::sync::atomic::Ordering::Relaxed),
        "api" => true,
        _ => false,
    };

    Json(serde_json::json!({
        "Method": Method,
        "Ready": Ready,
        "Error": Fault,
    }))
}

pub async fn GetPerformance(
    State(AppState): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    Json(crate::perf::Snapshot(&AppState))
}

pub async fn GetSettings(
    State(AppState): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "ScrobbleMethod": AppState.Db.GetScrobbleMethod(),
        "ScrobbleInLessons": AppState.Db.IsScrobbleInLessons(),
        "LastfmApiKey": AppState.Db.GetLastfmApiKey(),
        "LastfmUsername": AppState.Db.GetLastfmUsername(),
        "LastfmError": AppState.LastfmError.lock().unwrap().clone(),
        "LastfmVerified": AppState.Db.IsLastfmVerified(),
        "ApiKey": AppState.Db.GetApiKey(),
        "System": {
            "Version": env!("CARGO_PKG_VERSION"),
            "Address": _LanAddress(),
            "StartedAt": crate::STARTED_AT.get().copied().unwrap_or(0),
            "DatabaseBytes": _DatabaseBytes(),
        },
    }))
}

fn _LanAddress() -> Option<String> {
    const VIRTUAL: [&str; 11] = ["docker", "br-", "veth", "virbr", "vmnet", "vboxnet", "tun", "tap", "wg", "zt", "tailscale"];

    let List = if_addrs::get_if_addrs().ok()?;
    let mut Best: Option<(u8, std::net::Ipv4Addr)> = None;

    for Iface in List {
        if Iface.is_loopback() {
            continue;
        }
        let std::net::IpAddr::V4(Ip) = Iface.addr.ip() else { continue };
        if Ip.is_link_local() {
            continue;
        }
        let Rank = if VIRTUAL.iter().any(|Prefix| Iface.name.starts_with(Prefix)) {
            2
        } else if Ip.is_private() {
            0
        } else {
            1
        };
        if Best.is_none_or(|(Held, _)| Rank < Held) {
            Best = Some((Rank, Ip));
        }
    }

    Best.map(|(_, Ip)| format!("http://{}:9999", Ip))
}

fn _DatabaseBytes() -> u64 {
    let Main = crate::DatabasePath();
    let Wal = Main.with_file_name("zstio-tv.db-wal");
    [Main, Wal]
        .iter()
        .filter_map(|Path| std::fs::metadata(Path).ok())
        .map(|Meta| Meta.len())
        .sum()
}

#[derive(serde::Deserialize)]
pub(crate) struct ChangePasswordRequest {
    #[serde(rename = "Current")]
    current: String,
    #[serde(rename = "Next")]
    next: String,
}

pub async fn ChangePassword(
    State(AppState): State<Arc<AppState>>,
    Headers: HeaderMap,
    Json(Req): Json<ChangePasswordRequest>,
) -> Response {
    if Req.next.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "Error": "empty" }))).into_response();
    }

    if !AppState.Db.CheckPassword(&Req.current) {
        log!(Server, Warn, "password change rejected, wrong current password");
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({ "Error": "wrong" }))).into_response();
    }

    let Token = Headers
        .get(header::AUTHORIZATION)
        .and_then(|V| V.to_str().ok())
        .and_then(|V| V.strip_prefix("Bearer "))
        .unwrap_or("")
        .trim()
        .to_string();

    AppState.Db.SetPassword(&Req.next);
    AppState.KeepOnlySession(&Token);
    log!(Server, Info, "panel password changed");
    StatusCode::OK.into_response()
}

#[derive(serde::Deserialize)]
pub(crate) struct ScrobbleMethodRequest {
    #[serde(rename = "Method")]
    method: String,
}

pub async fn SetScrobbleMethod(
    State(AppState): State<Arc<AppState>>,
    Json(Req): Json<ScrobbleMethodRequest>,
) -> StatusCode {
    match Req.method.as_str() {
        "lastfm" | "spotify" | "serial" | "api" => {
            AppState.Db.SetScrobbleMethod(&Req.method);
            AppState.BumpSerial();
            StatusCode::OK
        }
        _ => StatusCode::BAD_REQUEST,
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct ScrobbleInLessonsRequest {
    #[serde(rename = "Enabled")]
    enabled: bool,
}

pub async fn SetScrobbleInLessons(
    State(AppState): State<Arc<AppState>>,
    Json(Req): Json<ScrobbleInLessonsRequest>,
) -> StatusCode {
    AppState.Db.SetScrobbleInLessons(Req.enabled);
    AppState.NotifyScreen();

    log!(Server, Info, "scrobbling during lessons: {}", Req.enabled);
    StatusCode::OK
}

#[derive(serde::Deserialize)]
pub(crate) struct LastfmRequest {
    #[serde(rename = "ApiKey")]
    api_key: String,
    #[serde(rename = "Username")]
    username: String,
}

#[derive(serde::Serialize)]
struct SaveLastfmResponse {
    #[serde(rename = "Ok")]
    ok: bool,
    #[serde(rename = "Error", skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

pub async fn SaveLastfm(
    State(AppState): State<Arc<AppState>>,
    Json(Req): Json<LastfmRequest>,
) -> impl IntoResponse {
    let ApiKey = Req.api_key.trim();
    let Username = Req.username.trim();

    if ApiKey.is_empty() || Username.is_empty() {
        return (StatusCode::OK, Json(SaveLastfmResponse { ok: false, error: Some("missing".into()) }));
    }

    match crate::lastfm::Verify(ApiKey, Username).await {
        Ok(()) => {
            AppState.Db.SetLastfmApiKey(ApiKey);
            AppState.Db.SetLastfmUsername(Username);
            AppState.Db.SetLastfmVerified(true);
            AppState.SetLastfmError(None);
            log!(Server, Info, "last.fm verified for {}", Username);
            (StatusCode::OK, Json(SaveLastfmResponse { ok: true, error: None }))
        }
        Err(F) => {
            AppState.Db.SetLastfmVerified(false);
            log!(Server, Warn, "last.fm verify failed for {}: {}", Username, F.Detail);
            (StatusCode::OK, Json(SaveLastfmResponse { ok: false, error: Some(F.Code) }))
        }
    }
}

pub async fn RegenerateApiKey(
    State(AppState): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let Key = AppState.Db.RegenerateApiKey();
    Json(serde_json::json!({ "ApiKey": Key }))
}

pub async fn NukeDatabase(
    State(AppState): State<Arc<AppState>>,
) -> StatusCode {
    AppState.Db.Nuke();
    AppState.DropSessions();
    AppState.SetTrack(None);
    log!(Server, Warn, "database nuked via panel, restarting");
    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        std::process::exit(0);
    });
    StatusCode::OK
}
