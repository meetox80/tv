#![allow(non_snake_case)]

use std::sync::Arc;
use axum::{extract::State, http::StatusCode, Json};

use crate::{AppState, SetupStage};
use crate::log::{Subsystem::Server, Level::*};
use super::PasswordRequest;

pub async fn SetPassword(
    State(AppState): State<Arc<AppState>>,
    Json(Req): Json<PasswordRequest>,
) -> StatusCode {
    if Req.password.is_empty() {
        return StatusCode::BAD_REQUEST;
    }
    AppState.Db.SetPassword(&Req.password);
    AppState.Db.MarkSetupComplete();
    AppState.SetSetupStage(SetupStage::Done);
    log!(Server, Info, "password set, setup complete");
    AppState.SetupShutdown.notify_one();
    StatusCode::OK
}

pub async fn UseLocal(State(AppState): State<Arc<AppState>>) -> StatusCode {
    for Key in ["tunnel_token", "management_hostname", "voting_hostname"] {
        AppState.Db.Set(Key, "");
    }
    AppState.SetSetupStage(SetupStage::Password);
    log!(Server, Info, "tunnel skipped, local only");
    StatusCode::OK
}

pub async fn Connect(State(AppState): State<Arc<AppState>>) -> StatusCode {
    AppState.ClientConnected.store(true, std::sync::atomic::Ordering::Relaxed);
    StatusCode::OK
}
