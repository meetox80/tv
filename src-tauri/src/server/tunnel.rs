#![allow(non_snake_case)]

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use axum::{extract::{ConnectInfo, State}, http::{HeaderMap, StatusCode}, response::{IntoResponse, Response}, Json};
use serde_json::json;

use crate::AppState;
use crate::cloudflared::NormalizeHostname;
use crate::server::traffic::Peer as Remote;
use crate::log::{Subsystem::Server, Level::*};

fn _Local(Peer: &SocketAddr, Headers: &HeaderMap) -> bool {
    if Headers.contains_key("cf-connecting-ip") || Headers.contains_key("cf-ray") {
        return false;
    }
    match Peer.ip() {
        IpAddr::V4(V4) => V4.is_loopback() || V4.is_private() || V4.is_link_local(),
        IpAddr::V6(V6) => match V6.to_ipv4_mapped() {
            Some(V4) => V4.is_loopback() || V4.is_private() || V4.is_link_local(),
            None => V6.is_loopback() || V6.is_unique_local() || V6.is_unicast_link_local(),
        },
    }
}

fn _Valid(Host: &str) -> bool {
    Host.len() <= 253
        && Host.contains('.')
        && Host.split('.').all(|Label| {
            !Label.is_empty()
                && Label.len() <= 63
                && !Label.starts_with('-')
                && !Label.ends_with('-')
                && Label.bytes().all(|B| B.is_ascii_lowercase() || B.is_ascii_digit() || B == b'-')
        })
}

async fn _Reaches(Host: &str, Expected: &str) -> bool {
    let Client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(6))
        .redirect(reqwest::redirect::Policy::none())
        .build()
    {
        Ok(C) => C,
        Err(E) => { fail!(Server, E, "could not build tunnel probe client"); return false; }
    };

    match Client.get(format!("https://{}/_whoami", Host)).send().await {
        Ok(Res) => Res.text().await.is_ok_and(|Body| Body.trim() == Expected),
        Err(E) => {
            log!(Server, Info, "tunnel probe {} failed: {}", Host, E);
            false
        }
    }
}

pub async fn GetTunnel(
    State(AppState): State<Arc<AppState>>,
    ConnectInfo(Remote(Peer)): ConnectInfo<Remote>,
    Headers: HeaderMap,
) -> Json<serde_json::Value> {
    Json(json!({
        "Management": NormalizeHostname(&AppState.Db.GetManagementHostname()),
        "Voting": NormalizeHostname(&AppState.Db.GetVotingHostname()),
        "Local": _Local(&Peer, &Headers),
    }))
}

#[derive(serde::Deserialize)]
pub(crate) struct TunnelRequest {
    #[serde(rename = "Kind")]
    kind: String,
    #[serde(rename = "Hostname")]
    hostname: String,
    #[serde(rename = "Force", default)]
    force: bool,
}

fn _Refuse(Code: &str) -> Response {
    Json(json!({ "Ok": false, "Error": Code })).into_response()
}

pub async fn SaveTunnel(
    State(AppState): State<Arc<AppState>>,
    ConnectInfo(Remote(Peer)): ConnectInfo<Remote>,
    Headers: HeaderMap,
    Json(Req): Json<TunnelRequest>,
) -> Response {
    let Management = match Req.kind.as_str() {
        "management" => true,
        "voting" => false,
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };

    if Management && !_Local(&Peer, &Headers) {
        log!(Server, Warn, "refused remote management hostname change from {}", Peer);
        return (StatusCode::FORBIDDEN, Json(json!({ "Ok": false, "Error": "remote" }))).into_response();
    }

    let Host = NormalizeHostname(&Req.hostname);
    if !_Valid(&Host) {
        return _Refuse("invalid");
    }

    let Other = NormalizeHostname(&if Management { AppState.Db.GetVotingHostname() } else { AppState.Db.GetManagementHostname() });
    if Host == Other {
        return _Refuse("same");
    }

    if !Req.force && !_Reaches(&Host, &AppState.Whoami(Management)).await {
        return _Refuse("unreachable");
    }

    AppState.Db.Set(if Management { "management_hostname" } else { "voting_hostname" }, &Host);
    log!(Server, Info, "{} hostname set to {}{}", if Management { "management" } else { "voting" }, Host, if Req.force { " (unverified)" } else { "" });

    if !Management {
        AppState.NotifyVoting();
    }

    Json(json!({ "Ok": true, "Hostname": Host })).into_response()
}
