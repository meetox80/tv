#![allow(non_snake_case)]

use std::collections::HashMap;
use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::AppState;
use crate::metadata;
use crate::voting::{self, BanOutcome, SecretCheck, Status};
use crate::log::{Subsystem::Voting, Level::*};

fn _Error(Status: StatusCode, Code: &str) -> Response {
    (Status, Json(json!({ "Error": Code }))).into_response()
}

pub async fn Overview(State(AppState): State<Arc<AppState>>) -> Response {
    let Host = crate::cloudflared::NormalizeHostname(&AppState.Db.GetVotingHostname());
    let mut Body = json!({
        "Enabled": AppState.Db.IsVotingEnabled(),
        "SiteKey": AppState.Db.GetTurnstileSiteKey(),
        "HasSecret": !AppState.Db.GetTurnstileSecret().is_empty(),
        "Url": (!Host.is_empty()).then(|| format!("https://{}", Host)),
        "Source": metadata::ActiveSource(&AppState.Db).AsText(),
    });

    if let (Some(Target), serde_json::Value::Object(Extra)) = (Body.as_object_mut(), voting::Counts(&AppState.Db)) {
        Target.extend(Extra);
    }

    Json(Body).into_response()
}

#[derive(serde::Deserialize)]
pub struct EnabledRequest {
    #[serde(rename = "Enabled")]
    enabled: bool,
}

pub async fn SetEnabled(
    State(AppState): State<Arc<AppState>>,
    Json(Req): Json<EnabledRequest>,
) -> Response {
    let Configured = !AppState.Db.GetTurnstileSiteKey().is_empty() && !AppState.Db.GetTurnstileSecret().is_empty();
    if Req.enabled && !Configured {
        return _Error(StatusCode::BAD_REQUEST, "captcha_unconfigured");
    }

    AppState.Db.SetVotingEnabled(Req.enabled);
    AppState.NotifyVoting();
    log!(Voting, Info, "voting {}", if Req.enabled { "enabled" } else { "disabled" });
    StatusCode::OK.into_response()
}

#[derive(serde::Deserialize)]
pub struct TurnstileRequest {
    #[serde(rename = "SiteKey")]
    site_key: String,
    #[serde(rename = "Secret", default)]
    secret: String,
}

fn _KeyShape(Value: &str) -> bool {
    (8..=128).contains(&Value.len())
        && Value.bytes().all(|B| B.is_ascii_alphanumeric() || B == b'_' || B == b'-')
}

pub async fn SaveTurnstile(
    State(AppState): State<Arc<AppState>>,
    Json(Req): Json<TurnstileRequest>,
) -> Response {
    let SiteKey = Req.site_key.trim();
    let Secret = Req.secret.trim();

    if !_KeyShape(SiteKey) {
        return _Error(StatusCode::BAD_REQUEST, "site_key");
    }

    if !Secret.is_empty() && !_KeyShape(Secret) {
        return _Error(StatusCode::BAD_REQUEST, "secret");
    }

    let Stored = AppState.Db.GetTurnstileSecret();
    let Candidate = if Secret.is_empty() { Stored.as_str() } else { Secret };
    if Candidate.is_empty() {
        return _Error(StatusCode::BAD_REQUEST, "secret");
    }

    match voting::CheckSecret(Candidate).await {
        SecretCheck::Valid => {}
        SecretCheck::Invalid => return _Error(StatusCode::BAD_REQUEST, "secret_rejected"),
        SecretCheck::Unreachable => return _Error(StatusCode::BAD_GATEWAY, "verify_unreachable"),
    }

    AppState.Db.SetTurnstile(SiteKey, (!Secret.is_empty()).then_some(Secret));
    log!(Voting, Info, "turnstile keys saved");
    Json(json!({ "Status": SecretCheck::Valid.AsText() })).into_response()
}

pub async fn CheckTurnstile(State(AppState): State<Arc<AppState>>) -> Response {
    let SiteKey = AppState.Db.GetTurnstileSiteKey();
    let Secret = AppState.Db.GetTurnstileSecret();
    if SiteKey.is_empty() || Secret.is_empty() {
        return Json(json!({ "Status": "unconfigured" })).into_response();
    }

    let Result = voting::CheckSecret(&Secret).await;
    if Result == SecretCheck::Invalid {
        log!(Voting, Warn, "stored turnstile secret was rejected by cloudflare");
    }
    Json(json!({ "Status": Result.AsText() })).into_response()
}

pub async fn ClearTurnstile(State(AppState): State<Arc<AppState>>) -> Response {
    AppState.Db.ClearTurnstile();
    AppState.NotifyVoting();
    log!(Voting, Info, "turnstile keys cleared, voting disabled");
    StatusCode::OK.into_response()
}

pub async fn ListSongs(
    State(AppState): State<Arc<AppState>>,
    Query(Params): Query<HashMap<String, String>>,
) -> Response {
    let Wanted = Params.get("status").map(String::as_str).unwrap_or("pending");
    match Status::Parse(Wanted) {
        Some(S) => Json(voting::AdminSongs(&AppState.Db, S)).into_response(),
        None => _Error(StatusCode::BAD_REQUEST, "status"),
    }
}

#[derive(serde::Deserialize)]
pub struct StatusRequest {
    #[serde(rename = "Status")]
    status: String,
}

pub async fn SetSongStatus(
    State(AppState): State<Arc<AppState>>,
    Path(Id): Path<i64>,
    Json(Req): Json<StatusRequest>,
) -> Response {
    let Next = match Status::Parse(&Req.status) {
        Some(S) => S,
        None => return _Error(StatusCode::BAD_REQUEST, "status"),
    };

    if !voting::SetStatus(&AppState.Db, Id, Next) {
        return _Error(StatusCode::NOT_FOUND, "not_found");
    }

    AppState.NotifyVoting();
    log!(Voting, Info, "song {} -> {}", Id, Next.AsText());

    match voting::AdminSong(&AppState.Db, Id) {
        Some(Song) => Json(Song).into_response(),
        None => _Error(StatusCode::NOT_FOUND, "not_found"),
    }
}

pub async fn Preview(
    State(AppState): State<Arc<AppState>>,
    Headers: HeaderMap,
    Path(Id): Path<i64>,
) -> Response {
    match crate::preview::Load(&AppState.Db, Id, false).await {
        Some(Clip) => super::vote::ServeClip(Clip, &Headers),
        None => _Error(StatusCode::NOT_FOUND, "no_preview"),
    }
}

pub async fn DeleteSong(
    State(AppState): State<Arc<AppState>>,
    Path(Id): Path<i64>,
) -> Response {
    if !voting::DeleteSong(&AppState.Db, Id) {
        return _Error(StatusCode::NOT_FOUND, "not_found");
    }

    AppState.NotifyVoting();
    log!(Voting, Info, "song {} deleted", Id);
    StatusCode::OK.into_response()
}

pub async fn ListBans(State(AppState): State<Arc<AppState>>) -> Response {
    Json(voting::BannedDevices(&AppState.Db)).into_response()
}

#[derive(serde::Deserialize)]
pub struct BanRequest {
    #[serde(rename = "Handle")]
    handle: String,
    #[serde(rename = "Banned")]
    banned: bool,
    #[serde(rename = "Reason", default)]
    reason: Option<String>,
}

pub async fn SetBan(
    State(AppState): State<Arc<AppState>>,
    Json(Req): Json<BanRequest>,
) -> Response {
    let Prefix = match voting::ParseHandle(&Req.handle) {
        Some(P) => P,
        None => return _Error(StatusCode::NOT_FOUND, "not_found"),
    };

    match voting::SetBan(&AppState.Db, &Prefix, Req.banned, Req.reason.as_deref()) {
        BanOutcome::Done(Device) => {
            AppState.NotifyVoting();
            log!(Voting, Info, "device {} {}", Prefix, if Req.banned { "banned" } else { "unbanned" });
            Json(Device).into_response()
        }
        BanOutcome::NotFound => _Error(StatusCode::NOT_FOUND, "not_found"),
        BanOutcome::Ambiguous => _Error(StatusCode::CONFLICT, "ambiguous"),
    }
}

pub async fn Art(
    State(AppState): State<Arc<AppState>>,
    Path(Key): Path<String>,
) -> Response {
    super::vote::ServeArt(&AppState, &Key).await
}
