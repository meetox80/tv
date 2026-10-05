#![allow(non_snake_case)]

use std::sync::Arc;
use std::time::Duration;
use axum::{
    extract::{Path, Query, Request, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::AppState;
use crate::metadata;
use crate::voting::{self, Device, RequestOutcome};
use crate::log::{Subsystem::Voting, Level::*};

const CSP: &str = "default-src 'self'; script-src 'self' https://challenges.cloudflare.com; \
frame-src https://challenges.cloudflare.com; style-src 'self' 'unsafe-inline'; img-src 'self' data:; \
font-src 'self'; connect-src 'self'; manifest-src 'self'; base-uri 'none'; form-action 'none'; \
frame-ancestors 'none'; object-src 'none'";

const MINUTE: Duration = Duration::from_secs(60);
const DAY: Duration = Duration::from_secs(24 * 60 * 60);

fn _Error(Status: StatusCode, Code: &str) -> Response {
    (Status, Json(json!({ "Error": Code }))).into_response()
}

fn _SlowDown(RetryAfter: u64) -> Response {
    (
        StatusCode::TOO_MANY_REQUESTS,
        [(header::RETRY_AFTER, RetryAfter.to_string())],
        Json(json!({ "Error": "slow_down", "RetryAfter": RetryAfter })),
    ).into_response()
}

fn _Ip(Headers: &HeaderMap) -> Option<String> {
    Headers
        .get("cf-connecting-ip")
        .and_then(|V| V.to_str().ok())
        .map(str::trim)
        .filter(|V| !V.is_empty() && V.len() <= 64 && V.parse::<std::net::IpAddr>().is_ok())
        .map(str::to_string)
}

fn _Limit(AppState: &AppState, Kind: &str, Subject: &str, Max: u32, Window: Duration) -> Result<(), Response> {
    AppState.Voting.Limits
        .Hit(&format!("{}:{}", Kind, Subject), Max, Window)
        .map_err(|Wait| {
            AppState.Voting.Activity.Throttled();
            _SlowDown(Wait)
        })
}

fn _Network(AppState: &AppState, Headers: &HeaderMap) -> String {
    AppState.Voting.NetworkKey(_Ip(Headers).as_deref().unwrap_or("unknown"))
}

fn _CookieToken(Headers: &HeaderMap) -> Option<String> {
    Headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|V| V.to_str().ok())
        .flat_map(|V| V.split(';'))
        .filter_map(|Pair| Pair.trim().split_once('='))
        .find(|(Name, _)| *Name == voting::COOKIE)
        .map(|(_, Value)| Value.trim().to_string())
        .filter(|T| voting::ValidToken(T))
}

fn _Device(AppState: &AppState, Headers: &HeaderMap) -> Option<Device> {
    let Token = _CookieToken(Headers)?;
    let Id = AppState.Voting.DeviceId(&AppState.Db, &Token);
    voting::FindDevice(&AppState.Db, &Id)
}

fn _RequireDevice(AppState: &AppState, Headers: &HeaderMap) -> Result<Device, Response> {
    match _Device(AppState, Headers) {
        Some(D) if D.Banned => Err(_Error(StatusCode::FORBIDDEN, "banned")),
        Some(D) => Ok(D),
        None => Err(_Error(StatusCode::UNAUTHORIZED, "device")),
    }
}

fn _RequireOpen(AppState: &AppState) -> Result<(), Response> {
    if AppState.Db.IsVotingEnabled() {
        Ok(())
    } else {
        Err(_Error(StatusCode::SERVICE_UNAVAILABLE, "closed"))
    }
}

fn _ExpectedOrigin(AppState: &AppState) -> Option<String> {
    let Host = crate::cloudflared::NormalizeHostname(&AppState.Db.GetVotingHostname());
    (!Host.is_empty()).then(|| format!("https://{}", Host))
}

pub async fn Guard(State(AppState): State<Arc<AppState>>, Req: Request, Next: Next) -> Response {
    if Req.method() != Method::GET && Req.method() != Method::HEAD {
        let Headers = Req.headers();

        let Marked = Headers
            .get("x-requested-with")
            .and_then(|V| V.to_str().ok())
            == Some("zstio-vote");

        let IsJson = Headers
            .get(header::CONTENT_TYPE)
            .and_then(|V| V.to_str().ok())
            .is_some_and(|V| V.starts_with("application/json"));

        let OriginOk = match _ExpectedOrigin(&AppState) {
            Some(Expected) => Headers
                .get(header::ORIGIN)
                .and_then(|V| V.to_str().ok())
                .is_some_and(|O| O.eq_ignore_ascii_case(&Expected)),
            None => true,
        };

        if !Marked || !IsJson || !OriginOk {
            log!(Voting, Warn, "rejected cross-site write to {}", Req.uri().path());
            return _Error(StatusCode::FORBIDDEN, "origin");
        }
    }

    let IsApi = Req.uri().path().starts_with("/api/");
    let mut Res = Next.run(Req).await;
    let H = Res.headers_mut();

    H.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static(CSP));
    H.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    H.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    H.insert(header::REFERRER_POLICY, HeaderValue::from_static("no-referrer"));
    H.insert(header::STRICT_TRANSPORT_SECURITY, HeaderValue::from_static("max-age=31536000"));
    H.insert("cross-origin-opener-policy", HeaderValue::from_static("same-origin"));
    H.insert("cross-origin-resource-policy", HeaderValue::from_static("same-origin"));
    H.insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=(), payment=(), usb=(), interest-cohort=()"),
    );
    if IsApi && !H.contains_key(header::CACHE_CONTROL) {
        H.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    }

    Res
}

pub async fn GetState(State(AppState): State<Arc<AppState>>, Headers: HeaderMap) -> Response {
    let SiteKey = AppState.Db.GetTurnstileSiteKey();
    let Configured = !SiteKey.is_empty() && !AppState.Db.GetTurnstileSecret().is_empty();
    let Device = _Device(&AppState, &Headers).map(|D| voting::DeviceJson(&AppState.Db, &D));

    Json(json!({
        "Enabled": AppState.Db.IsVotingEnabled(),
        "SiteKey": Configured.then_some(SiteKey),
        "Limit": voting::DAILY_LIMIT,
        "ResetIn": voting::SecondsToReset(&AppState.Db),
        "Device": Device,
    })).into_response()
}

#[derive(serde::Deserialize)]
pub struct DeviceRequest {
    #[serde(rename = "Captcha")]
    captcha: String,
}

pub async fn RegisterDevice(
    State(AppState): State<Arc<AppState>>,
    Headers: HeaderMap,
    Json(Body): Json<DeviceRequest>,
) -> Response {
    if let Err(R) = _RequireOpen(&AppState) {
        return R;
    }

    if AppState.Db.GetTurnstileSecret().is_empty() || AppState.Db.GetTurnstileSiteKey().is_empty() {
        return _Error(StatusCode::SERVICE_UNAVAILABLE, "captcha_unconfigured");
    }

    let Network = _Network(&AppState, &Headers);
    if let Err(R) = _Limit(&AppState, "captcha", &Network, 40, MINUTE * 10) {
        return R;
    }

    let Ip = _Ip(&Headers);
    if let Err(Reason) = voting::VerifyCaptcha(&AppState.Db, &Body.captcha, Ip.as_deref()).await {
        log!(Voting, Warn, "captcha rejected: {}", Reason);
        return _Error(StatusCode::BAD_REQUEST, "captcha");
    }

    if let Err(R) = _Limit(&AppState, "device", &Network, 200, DAY) {
        log!(Voting, Warn, "device creation throttled for network {}", &Network[..8]);
        return R;
    }

    let Token = voting::NewToken();
    let Id = AppState.Voting.DeviceId(&AppState.Db, &Token);
    voting::CreateDevice(&AppState.Db, &Id);

    let Device = match voting::FindDevice(&AppState.Db, &Id) {
        Some(D) => D,
        None => return _Error(StatusCode::INTERNAL_SERVER_ERROR, "device"),
    };

    log!(Voting, Info, "device registered {}", voting::Handle(&Id));

    let Cookie = format!(
        "{}={}; Path=/; Max-Age={}; Secure; HttpOnly; SameSite=Strict",
        voting::COOKIE,
        Token,
        365 * 24 * 60 * 60
    );

    (
        StatusCode::OK,
        [(header::SET_COOKIE, Cookie)],
        Json(voting::DeviceJson(&AppState.Db, &Device)),
    ).into_response()
}

#[derive(serde::Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
}

pub async fn Search(
    State(AppState): State<Arc<AppState>>,
    Headers: HeaderMap,
    Query(Params): Query<SearchQuery>,
) -> Response {
    if let Err(R) = _RequireOpen(&AppState) {
        return R;
    }

    let Raw = Params.q.unwrap_or_default();
    let Text = Raw.trim();
    let Length = Text.chars().count();
    if !(2..=100).contains(&Length) || Text.chars().any(char::is_control) {
        return _Error(StatusCode::BAD_REQUEST, "query");
    }

    let Network = _Network(&AppState, &Headers);
    if let Err(R) = _Limit(&AppState, "search", &Network, 300, MINUTE) {
        return R;
    }
    if let Some(D) = _Device(&AppState, &Headers) {
        if D.Banned {
            return _Error(StatusCode::FORBIDDEN, "banned");
        }
        if let Err(R) = _Limit(&AppState, "search-device", &D.Id, 30, MINUTE) {
            return R;
        }
    }

    let Started = std::time::Instant::now();
    let Found = metadata::Search(&AppState.Db, Text).await;
    AppState.Voting.Activity.Searched(Started.elapsed(), Found.is_ok());

    match Found {
        Ok((Source, Tracks)) => {
            let Results: Vec<serde_json::Value> = Tracks
                .iter()
                .map(|T| voting::TrackJson(&AppState.Db, T))
                .collect();
            Json(json!({ "Source": Source.AsText(), "Results": Results })).into_response()
        }
        Err(E) => {
            log!(Voting, Error, "search failed everywhere: {}", E);
            _Error(StatusCode::BAD_GATEWAY, "upstream")
        }
    }
}

#[derive(serde::Deserialize)]
pub struct SongRequest {
    #[serde(rename = "Provider")]
    provider: String,
    #[serde(rename = "Id")]
    id: String,
}

pub async fn CreateRequest(
    State(AppState): State<Arc<AppState>>,
    Headers: HeaderMap,
    Json(Body): Json<SongRequest>,
) -> Response {
    if let Err(R) = _RequireOpen(&AppState) {
        return R;
    }

    let Device = match _RequireDevice(&AppState, &Headers) {
        Ok(D) => D,
        Err(R) => return R,
    };

    if let Err(R) = _Limit(&AppState, "request", &Device.Id, 6, MINUTE) {
        return R;
    }

    if voting::RequestsToday(&AppState.Db, &Device.Id) >= voting::DAILY_LIMIT {
        return (StatusCode::TOO_MANY_REQUESTS, Json(json!({ "Error": "limit", "Left": 0 }))).into_response();
    }

    let Provider = match metadata::Provider::Parse(&Body.provider) {
        Some(P) => P,
        None => return _Error(StatusCode::NOT_FOUND, "not_found"),
    };

    if Body.id.is_empty() || Body.id.len() > 600 {
        return _Error(StatusCode::NOT_FOUND, "not_found");
    }

    let Track = match metadata::Resolve(&AppState.Db, Provider, &Body.id).await {
        Some(T) => T,
        None => return _Error(StatusCode::NOT_FOUND, "not_found"),
    };

    match voting::CreateRequest(&AppState.Db, &Device.Id, &Track) {
        RequestOutcome::Created(Song) => {
            AppState.Voting.Activity.Requested();
            log!(Voting, Info, "request from {}: {} - {}", voting::Handle(&Device.Id), Track.Artist, Track.Title);
            let Left = (voting::DAILY_LIMIT - voting::RequestsToday(&AppState.Db, &Device.Id)).max(0);
            (StatusCode::CREATED, Json(json!({ "Song": Song, "Left": Left }))).into_response()
        }
        RequestOutcome::Duplicate(Status) => {
            (StatusCode::CONFLICT, Json(json!({ "Error": "duplicate", "Status": Status }))).into_response()
        }
        RequestOutcome::Limit => {
            (StatusCode::TOO_MANY_REQUESTS, Json(json!({ "Error": "limit", "Left": 0 }))).into_response()
        }
    }
}

pub async fn Songs(State(AppState): State<Arc<AppState>>, Headers: HeaderMap) -> Response {
    let Network = _Network(&AppState, &Headers);
    if let Err(R) = _Limit(&AppState, "songs", &Network, 2000, MINUTE) {
        return R;
    }

    let Device = _Device(&AppState, &Headers).filter(|D| !D.Banned);
    Json(voting::Board(&AppState.Db, Device.as_ref().map(|D| D.Id.as_str()))).into_response()
}

#[derive(serde::Deserialize)]
pub struct VoteRequest {
    #[serde(rename = "Value")]
    value: i64,
}

pub async fn CastVote(
    State(AppState): State<Arc<AppState>>,
    Headers: HeaderMap,
    Path(Id): Path<i64>,
    Json(Body): Json<VoteRequest>,
) -> Response {
    if let Err(R) = _RequireOpen(&AppState) {
        return R;
    }

    if !(-1..=1).contains(&Body.value) {
        return _Error(StatusCode::BAD_REQUEST, "value");
    }

    let Device = match _RequireDevice(&AppState, &Headers) {
        Ok(D) => D,
        Err(R) => return R,
    };

    if let Err(R) = _Limit(&AppState, "vote", &Device.Id, 60, MINUTE) {
        return R;
    }

    match voting::Vote(&AppState.Db, &Device.Id, Id, Body.value) {
        Some((Score, Vote)) => {
            AppState.Voting.Activity.Voted();
            AppState.NotifyVoting();
            Json(json!({ "Score": Score, "Vote": Vote })).into_response()
        }
        None => _Error(StatusCode::NOT_FOUND, "not_found"),
    }
}

pub async fn ServeArt(AppState: &AppState, Key: &str) -> Response {
    let Stored = voting::ArtUrl(&AppState.Db, Key);
    match metadata::Artwork(&AppState.Db, Key, Stored).await {
        Some((Bytes, Mime)) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, Mime),
                (header::CACHE_CONTROL, "public, max-age=604800, immutable".to_string()),
            ],
            Bytes,
        ).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn Art(
    State(AppState): State<Arc<AppState>>,
    Headers: HeaderMap,
    Path(Key): Path<String>,
) -> Response {
    let Network = _Network(&AppState, &Headers);
    if let Err(R) = _Limit(&AppState, "art", &Network, 4000, MINUTE) {
        return R;
    }
    ServeArt(&AppState, &Key).await
}

fn _Span(Value: &str, Total: usize) -> Result<Option<(usize, usize)>, ()> {
    let Some(Spec) = Value.trim().strip_prefix("bytes=") else { return Ok(None) };
    if Spec.contains(',') || Total == 0 {
        return Ok(None);
    }
    let Some((From, To)) = Spec.split_once('-') else { return Ok(None) };
    let (From, To) = (From.trim(), To.trim());

    let (Start, End) = if From.is_empty() {
        let Ok(Suffix) = To.parse::<usize>() else { return Ok(None) };
        if Suffix == 0 {
            return Err(());
        }
        (Total.saturating_sub(Suffix), Total - 1)
    } else {
        let Ok(Start) = From.parse::<usize>() else { return Ok(None) };
        let End = if To.is_empty() {
            Total - 1
        } else {
            let Ok(End) = To.parse::<usize>() else { return Ok(None) };
            End.min(Total - 1)
        };
        (Start, End)
    };

    if Start >= Total || Start > End {
        return Err(());
    }
    Ok(Some((Start, End)))
}

pub async fn Preview(
    State(AppState): State<Arc<AppState>>,
    Headers: HeaderMap,
    Path(Id): Path<i64>,
) -> Response {
    let Network = _Network(&AppState, &Headers);
    if let Err(R) = _Limit(&AppState, "preview", &Network, 600, MINUTE) {
        return R;
    }

    match crate::preview::Load(&AppState.Db, Id, true).await {
        Some(Clip) => ServeClip(Clip, &Headers),
        None => _Error(StatusCode::NOT_FOUND, "no_preview"),
    }
}

pub fn ServeClip(Clip: crate::preview::Clip, Headers: &HeaderMap) -> Response {
    let Total = Clip.Bytes.len();
    let Common = [
        (header::CONTENT_TYPE, Clip.Mime.clone()),
        (header::ACCEPT_RANGES, "bytes".to_string()),
        (header::CACHE_CONTROL, "private, max-age=86400".to_string()),
    ];

    let Span = match Headers.get(header::RANGE).and_then(|V| V.to_str().ok()) {
        Some(Value) => _Span(Value, Total),
        None => Ok(None),
    };

    match Span {
        Ok(None) => (StatusCode::OK, Common, Clip.Bytes).into_response(),
        Ok(Some((Start, End))) => (
            StatusCode::PARTIAL_CONTENT,
            Common,
            [(header::CONTENT_RANGE, format!("bytes {}-{}/{}", Start, End, Total))],
            Clip.Bytes.slice(Start..=End),
        ).into_response(),
        Err(()) => (
            StatusCode::RANGE_NOT_SATISFIABLE,
            [(header::CONTENT_RANGE, format!("bytes */{}", Total))],
        ).into_response(),
    }
}

pub async fn NotFound() -> Response {
    _Error(StatusCode::NOT_FOUND, "not_found")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_cover_common_ranges() {
        assert_eq!(_Span("bytes=0-", 100), Ok(Some((0, 99))));
        assert_eq!(_Span("bytes=0-1", 100), Ok(Some((0, 1))));
        assert_eq!(_Span("bytes=90-500", 100), Ok(Some((90, 99))));
        assert_eq!(_Span("bytes=-10", 100), Ok(Some((90, 99))));
        assert_eq!(_Span("bytes=100-", 100), Err(()));
        assert_eq!(_Span("bytes=0-1,5-6", 100), Ok(None));
        assert_eq!(_Span("items=0-1", 100), Ok(None));
    }
}
