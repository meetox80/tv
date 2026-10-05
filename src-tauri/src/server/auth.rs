#![allow(non_snake_case)]

use std::sync::Arc;
use axum::{extract::State, http::StatusCode, response::{IntoResponse, Response}, Json};

use crate::{AppState, SetupStage};
use crate::log::{Subsystem::Auth, Level::*};

#[derive(serde::Deserialize)]
pub struct AuthRequest {
    pub code: String,
}

pub async fn ValidateCode(
    State(AppState): State<Arc<AppState>>,
    Json(Body): Json<AuthRequest>,
) -> Response {
    let Matches = {
        let Code = AppState.PairingCode.lock().unwrap();
        Body.code == *Code
    };

    if !Matches {
        log!(Auth, Warn, "code rejected");
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let Token = AppState.CreateSession();
    log!(Auth, Info, "code accepted");
    AppState.CodeAccepted.store(true, std::sync::atomic::Ordering::Relaxed);
    AppState.SetSetupStage(SetupStage::Token);

    (StatusCode::OK, Json(serde_json::json!({ "Token": Token }))).into_response()
}

pub async fn CheckSession() -> StatusCode {
    StatusCode::OK
}
