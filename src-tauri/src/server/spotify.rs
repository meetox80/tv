#![allow(non_snake_case)]

use std::sync::Arc;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Json,
};

use crate::AppState;
use crate::spotify::{AuthorizeUrl, ExchangeCode, RedirectUri};
use crate::log::{Subsystem::Spotify, Level::*};

pub async fn GetStatus(State(AppState): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "ClientId": AppState.Db.GetSpotifyClientId(),
        "HasSecret": !AppState.Db.GetSpotifyClientSecret().is_empty(),
        "Connected": AppState.Db.IsSpotifyConnected(),
        "RedirectUri": RedirectUri(&AppState.Db),
    }))
}

#[derive(serde::Deserialize)]
pub(crate) struct CredentialsRequest {
    #[serde(rename = "ClientId")]
    client_id: String,
    #[serde(rename = "ClientSecret")]
    client_secret: String,
}

pub async fn SaveCredentials(
    State(AppState): State<Arc<AppState>>,
    Json(Req): Json<CredentialsRequest>,
) -> Response {
    let ClientId = Req.client_id.trim();
    let Stored = AppState.Db.GetSpotifyClientSecret();
    let ClientSecret = match Req.client_secret.trim() {
        "" => Stored.as_str(),
        Given => Given,
    };

    if ClientId.is_empty() || ClientSecret.is_empty() {
        return StatusCode::BAD_REQUEST.into_response();
    }

    AppState.Db.SetSpotifyCredentials(ClientId, ClientSecret);
    log!(Spotify, Info, "credentials saved");

    Json(serde_json::json!({ "RedirectUri": RedirectUri(&AppState.Db) })).into_response()
}

pub async fn Authorize(State(AppState): State<Arc<AppState>>) -> Response {
    let ClientId = AppState.Db.GetSpotifyClientId();

    if ClientId.is_empty() || AppState.Db.GetSpotifyClientSecret().is_empty() {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let Redir = RedirectUri(&AppState.Db);
    let OauthState = AppState.Db.NewSpotifyOauthState();

    log!(Spotify, Info, "authorize url issued for {}", Redir);
    Json(serde_json::json!({ "Url": AuthorizeUrl(&ClientId, &Redir, &OauthState) })).into_response()
}

#[derive(serde::Deserialize)]
pub(crate) struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

pub async fn Callback(
    State(AppState): State<Arc<AppState>>,
    Query(Params): Query<CallbackQuery>,
) -> Redirect {
    if let Some(Denied) = Params.error {
        log!(Spotify, Warn, "authorization denied: {}", Denied);
        return Redirect::to("/?spotify=denied");
    }

    let Expected = AppState.Db.TakeSpotifyOauthState();
    let Received = Params.state.unwrap_or_default();

    if Expected.is_empty() || Expected != Received {
        log!(Spotify, Warn, "callback state mismatch");
        return Redirect::to("/?spotify=state_mismatch");
    }

    let Code = match Params.code {
        Some(C) if !C.is_empty() => C,
        _ => return Redirect::to("/?spotify=no_code"),
    };

    let ClientId = AppState.Db.GetSpotifyClientId();
    let ClientSecret = AppState.Db.GetSpotifyClientSecret();
    let Redir = RedirectUri(&AppState.Db);

    match ExchangeCode(&ClientId, &ClientSecret, &Redir, &Code).await {
        Ok(RefreshToken) => {
            AppState.Db.SetSpotifyRefreshToken(&RefreshToken);
            AppState.Db.SetScrobbleMethod("spotify");
            log!(Spotify, Info, "connected");
            Redirect::to("/?spotify=connected")
        }
        Err(E) => {
            log!(Spotify, Error, "token exchange failed: {}", E);
            Redirect::to("/?spotify=exchange_failed")
        }
    }
}

pub async fn Disconnect(State(AppState): State<Arc<AppState>>) -> StatusCode {
    AppState.Db.DisconnectSpotify();

    if AppState.Db.GetScrobbleMethod() == "spotify" {
        AppState.SetTrack(None);
    }

    log!(Spotify, Info, "disconnected");
    StatusCode::OK
}
