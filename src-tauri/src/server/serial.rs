#![allow(non_snake_case)]

use std::sync::Arc;
use std::sync::atomic::Ordering;
use axum::{extract::State, http::StatusCode, Json, response::IntoResponse};

use crate::AppState;
use crate::serial;
use crate::log::{Subsystem::Server, Level::*};

fn _Status(AppState: &Arc<AppState>) -> serde_json::Value {
    let (Key, Text) = serial::EnsureKeypair(&AppState.Db);

    serde_json::json!({
        "Port": AppState.Db.GetSerialPort(),
        "Baud": AppState.Db.GetSerialBaud(),
        "Ports": serial::ListPorts(),
        "PublicKey": Text,
        "Fingerprint": serial::Fingerprint(&Key),
        "PeerKey": AppState.Db.GetSerialPeerKey(),
        "Open": AppState.SerialOpen.load(Ordering::Relaxed),
        "Error": AppState.SerialError.lock().unwrap().clone(),
    })
}

pub async fn GetStatus(State(AppState): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(_Status(&AppState))
}

#[derive(serde::Deserialize)]
pub(crate) struct SerialRequest {
    #[serde(rename = "Port")]
    port: String,
    #[serde(rename = "Baud")]
    baud: u32,
    #[serde(rename = "PeerKey")]
    peer_key: String,
}

#[derive(serde::Serialize)]
struct SaveResponse {
    #[serde(rename = "Ok")]
    ok: bool,
    #[serde(rename = "Error", skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

pub async fn SaveConfig(
    State(AppState): State<Arc<AppState>>,
    Json(Req): Json<SerialRequest>,
) -> impl IntoResponse {
    let Port = Req.port.trim();
    let Peer = Req.peer_key.trim();

    if Port.is_empty() {
        return (StatusCode::OK, Json(SaveResponse { ok: false, error: Some("no_port".into()) }));
    }

    if !(1200..=4_000_000).contains(&Req.baud) {
        return (StatusCode::OK, Json(SaveResponse { ok: false, error: Some("bad_baud".into()) }));
    }

    if serial::ParsePeerKey(Peer).is_none() {
        return (StatusCode::OK, Json(SaveResponse { ok: false, error: Some("bad_key".into()) }));
    }

    AppState.Db.SetSerial(Port, Req.baud);
    AppState.Db.SetSerialPeerKey(Peer);
    AppState.SetSerialError(None);
    AppState.BumpSerial();

    log!(Server, Info, "serial set to {} at {} baud", Port, Req.baud);
    (StatusCode::OK, Json(SaveResponse { ok: true, error: None }))
}

pub async fn RegenerateKeys(State(AppState): State<Arc<AppState>>) -> Json<serde_json::Value> {
    serial::Regenerate(&AppState.Db);
    AppState.BumpSerial();
    Json(_Status(&AppState))
}
