#![allow(non_snake_case)]

pub mod auth;
pub mod cloudflared;
mod docs;
mod extensions;
mod guard;
mod live;
mod panel;
mod screen;
mod serial;
mod setup;
mod slides;
mod spotify;
pub mod traffic;
mod tunnel;
mod vote;
mod voting;

use std::sync::Arc;
use axum::{Router, extract::DefaultBodyLimit, routing::{get, post}, http::StatusCode, response::{IntoResponse, Response}};
use rust_embed::RustEmbed;

use crate::AppState;

#[derive(RustEmbed)]
#[folder = "web/setup/dist/"]
struct SetupAssets;

#[derive(RustEmbed)]
#[folder = "web/panel/dist/"]
struct PanelAssets;

#[derive(RustEmbed)]
#[folder = "web/vote/dist/"]
struct VoteAssets;

const IMMUTABLE: &str = "public, max-age=31536000, immutable";
const REVALIDATE: &str = "no-cache";

fn _CacheControl(Path: &str) -> &'static str {
    if Path.starts_with("assets/") { IMMUTABLE } else { REVALIDATE }
}

fn _ServeEmbedded<E: RustEmbed>(Path: &str) -> Response {
    match E::get(Path) {
        Some(File) => {
            let Mime = File.metadata.mimetype();
            (
                StatusCode::OK,
                [
                    (axum::http::header::CONTENT_TYPE, Mime.to_string()),
                    (axum::http::header::CACHE_CONTROL, _CacheControl(Path).to_string()),
                ],
                File.data.to_vec(),
            ).into_response()
        }
        None => match E::get("index.html") {
            Some(Index) => (
                StatusCode::OK,
                [
                    (axum::http::header::CONTENT_TYPE, "text/html".to_string()),
                    (axum::http::header::CACHE_CONTROL, REVALIDATE.to_string()),
                ],
                Index.data.to_vec(),
            ).into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        },
    }
}

async fn Health() -> &'static str { "zstio-tv" }

#[derive(serde::Deserialize)]
pub(crate) struct PasswordRequest {
    #[serde(rename = "Password")]
    pub password: String,
}

pub fn _CreateRouter(AppState: Arc<AppState>) -> Router {
    let Guarded = Router::new()
        .route("/api/auth/check", post(auth::CheckSession))
        .route("/api/cloudflared", get(cloudflared::GetConfig).post(cloudflared::SaveConfig))
        .route("/api/cloudflared/verify", post(cloudflared::VerifyHostnames))
        .route("/api/setup/password", post(setup::SetPassword))
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&AppState),
            guard::RequireSession,
        ));

    let Public = Router::new()
        .route("/api/connect", get(setup::Connect))
        .route("/api/auth", post(auth::ValidateCode))
        .route("/_health", get(Health))
        .route("/", get(|| async { _ServeEmbedded::<SetupAssets>("index.html") }))
        .route("/{*path}", get(|axum::extract::Path(P): axum::extract::Path<String>| async move { _ServeEmbedded::<SetupAssets>(&P) }));

    Public.merge(Guarded).with_state(AppState)
}

pub fn _CreatePanelRouter(AppState: Arc<AppState>) -> Router {
    let Guarded = Router::new()
        .route("/api/playback/source", get(panel::GetPlaybackSource))
        .route("/api/settings", get(panel::GetSettings))
        .route("/api/performance", get(panel::GetPerformance))
        .route("/api/settings/scrobble-method", post(panel::SetScrobbleMethod))
        .route("/api/settings/scrobble-in-lessons", post(panel::SetScrobbleInLessons))
        .route("/api/settings/lastfm", post(panel::SaveLastfm))
        .route("/api/settings/regenerate-key", post(panel::RegenerateApiKey))
        .route("/api/settings/nuke", post(panel::NukeDatabase))
        .route("/api/settings/password", post(panel::ChangePassword))
        .route("/api/spotify/status", get(spotify::GetStatus))
        .route("/api/spotify/credentials", post(spotify::SaveCredentials))
        .route("/api/spotify/authorize", post(spotify::Authorize))
        .route("/api/spotify/disconnect", post(spotify::Disconnect))
        .route("/api/screen", get(screen::GetScreen))
        .route("/api/screen/weather", post(screen::SaveWeather))
        .route("/api/screen/notice", post(screen::SaveNotice))
        .route("/api/screen/pages", post(screen::SavePages))
        .route("/api/screen/live", get(screen::GetLive))
        .route("/api/screen/preview", get(screen::GetPreview))
        .route("/api/tunnel", get(tunnel::GetTunnel).post(tunnel::SaveTunnel))
        .route("/api/slides", get(slides::List).post(slides::Create))
        .route("/api/slides/assets", post(slides::Upload).layer(DefaultBodyLimit::max(crate::slides::ASSET_LIMIT + 1024)))
        .route("/api/slides/{id}", get(slides::Get).put(slides::Save).delete(slides::Delete).layer(DefaultBodyLimit::max(8 * 1024 * 1024)))
        .route("/api/extensions", get(extensions::List))
        .route("/api/extensions/disable", post(extensions::DisableAll))
        .route("/api/extensions/reload", post(extensions::Reload))
        .route("/api/extensions/{id}", post(extensions::Toggle))
        .route("/api/extensions/{id}/panel.js", get(extensions::PanelModule))
        .route("/api/screen/lessons", get(screen::GetLessons).post(screen::SaveLessons))
        .route("/api/serial", get(serial::GetStatus).post(serial::SaveConfig))
        .route("/api/serial/regenerate", post(serial::RegenerateKeys))
        .route("/api/voting", get(voting::Overview))
        .route("/api/voting/enabled", post(voting::SetEnabled))
        .route("/api/voting/turnstile", get(voting::CheckTurnstile).post(voting::SaveTurnstile).delete(voting::ClearTurnstile))
        .route("/api/voting/songs", get(voting::ListSongs))
        .route("/api/voting/songs/{id}/status", post(voting::SetSongStatus))
        .route("/api/voting/songs/{id}/preview", get(voting::Preview))
        .route("/api/voting/songs/{id}", axum::routing::delete(voting::DeleteSong))
        .route("/api/voting/devices", get(voting::ListBans).post(voting::SetBan))
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&AppState),
            guard::RequireSession,
        ));

    let Public = Router::new()
        .route("/_health", get(Health))
        .route("/_whoami", get(|axum::extract::State(S): axum::extract::State<Arc<AppState>>| async move { S.Whoami(true) }))
        .route("/safe", get(extensions::Safe))
        .route("/api/login", post(panel::Login))
        .route("/api/performance/ws", get(live::PerformanceSocket))
        .route("/api/playback/poke", post(panel::PlaybackPoke))
        .route("/api/lessons/poke", post(panel::LessonPoke))
        .route("/api/playback/current", get(panel::GetCurrentTrack))
        .route("/api/playback/cover", get(panel::GetCover).post(panel::PutCover))
        .route("/api/news/image", get(panel::GetNewsImage))
        .route("/api/slides/assets/{id}", get(slides::Asset))
        .route("/api/spotify/callback", get(spotify::Callback))
        .route("/api/voting/art/{key}", get(voting::Art))
        .route("/llms.txt", get(docs::LlmsTxt))
        .route("/", get(|| async { _ServeEmbedded::<PanelAssets>("index.html") }))
        .route("/{*path}", get(|axum::extract::Path(P): axum::extract::Path<String>| async move { _ServeEmbedded::<PanelAssets>(&P) }));

    Public
        .merge(Guarded)
        .layer(axum::middleware::from_fn_with_state(Arc::clone(&AppState.Traffic), traffic::CountRequest))
        .with_state(AppState)
}

pub fn _CreateVoteRouter(AppState: Arc<AppState>) -> Router {
    let Api = Router::new()
        .route("/api/state", get(vote::GetState))
        .route("/api/device", post(vote::RegisterDevice))
        .route("/api/search", get(vote::Search))
        .route("/api/requests", post(vote::CreateRequest))
        .route("/api/songs", get(vote::Songs))
        .route("/api/songs/{id}/vote", post(vote::CastVote))
        .route("/api/songs/{id}/preview", get(vote::Preview))
        .route("/api/art/{key}", get(vote::Art))
        .route("/api/{*rest}", axum::routing::any(vote::NotFound))
        .layer(DefaultBodyLimit::max(8 * 1024));

    Router::new()
        .merge(Api)
        .route("/_health", get(Health))
        .route("/_whoami", get(|axum::extract::State(S): axum::extract::State<Arc<AppState>>| async move { S.Whoami(false) }))
        .route("/", get(|| async { _ServeEmbedded::<VoteAssets>("index.html") }))
        .route("/{*path}", get(|axum::extract::Path(P): axum::extract::Path<String>| async move { _ServeEmbedded::<VoteAssets>(&P) }))
        .layer(axum::middleware::from_fn_with_state(Arc::clone(&AppState), vote::Guard))
        .layer(axum::middleware::from_fn_with_state(Arc::clone(&AppState.Traffic), traffic::CountRequest))
        .with_state(AppState)
}
