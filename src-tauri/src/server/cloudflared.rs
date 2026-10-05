#![allow(non_snake_case)]

use std::sync::Arc;
use axum::{extract::State, http::StatusCode, Json, response::IntoResponse};

use crate::{AppState, SetupStage};
use crate::log::{Subsystem::Server, Level::*};
use crate::cloudflared::{HostnameVerifyResult, _VerifyToken, _VerifyHostnames};

#[derive(serde::Deserialize)]
pub struct SaveConfigRequest {
    #[serde(rename = "TunnelToken")]
    tunnel_token: String,
    #[serde(rename = "ManagementHostname", default)]
    management_hostname: Option<String>,
    #[serde(rename = "VotingHostname", default)]
    voting_hostname: Option<String>,
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
    Json(Req): Json<SaveConfigRequest>,
) -> impl IntoResponse {
    log!(Server, Info, "POST /api/cloudflared token_len={}", Req.tunnel_token.len());

    let HasHostnames: bool = Req.management_hostname.is_some() && Req.voting_hostname.is_some();
    AppState.SetSetupStage(SetupStage::Verifying);

    AppState.Db.Set("tunnel_token", &Req.tunnel_token);

    if let Some(ref H) = Req.management_hostname {
        AppState.Db.Set("management_hostname", H);
    }
    if let Some(ref H) = Req.voting_hostname {
        AppState.Db.Set("voting_hostname", H);
    }

    let Bin = AppState.CloudflaredBin.lock().unwrap().clone();
    match _VerifyToken(&Bin, &Req.tunnel_token).await {
        Ok(()) => {
            AppState.SetSetupStage(if HasHostnames { SetupStage::Password } else { SetupStage::Ports });
            log!(Server, Info, "token verified");
            (StatusCode::OK, Json(SaveResponse { ok: true, error: None }))
        },
        Err(Msg) => {
            AppState.SetSetupStage(if HasHostnames { SetupStage::Ports } else { SetupStage::Token });
            log!(Server, Warn, "token verify failed: {}", Msg);
            (StatusCode::OK, Json(SaveResponse { ok: false, error: Some(Msg) }))
        },
    }
}

#[derive(serde::Serialize)]
pub struct ConfigResponse {
    #[serde(rename = "TunnelToken")]
    tunnel_token: String,
    #[serde(rename = "ManagementHostname")]
    management_hostname: String,
    #[serde(rename = "VotingHostname")]
    voting_hostname: String,
}

pub async fn GetConfig(
    State(AppState): State<Arc<AppState>>,
) -> Json<ConfigResponse> {
    Json(ConfigResponse {
        tunnel_token: AppState.Db.GetTunnelToken(),
        management_hostname: AppState.Db.GetManagementHostname(),
        voting_hostname: AppState.Db.GetVotingHostname(),
    })
}

#[derive(serde::Deserialize)]
pub struct VerifyHostnamesRequest {
    #[serde(rename = "ManagementHostname")]
    management_hostname: String,
    #[serde(rename = "VotingHostname")]
    voting_hostname: String,
}

pub async fn VerifyHostnames(
    State(AppState): State<Arc<AppState>>,
    Json(Req): Json<VerifyHostnamesRequest>,
) -> Json<HostnameVerifyResult> {
    log!(Server, Info, "POST /api/cloudflared/verify mgmt={} vote={}",
        Req.management_hostname, Req.voting_hostname);

    AppState.SetSetupStage(SetupStage::Verifying);

    let Token = AppState.Db.GetTunnelToken();
    let Bin = AppState.CloudflaredBin.lock().unwrap().clone();

    let Result = _VerifyHostnames(
        &Bin,
        &Token,
        &Req.management_hostname,
        &Req.voting_hostname,
    ).await;

    log!(Server, Info, "verify response: mgmt={} vote={} err={:?}",
        Result.management, Result.voting, Result.error);

    if !Result.management || !Result.voting || Result.error.is_some() {
        AppState.SetSetupStage(SetupStage::Ports);
    }

    Json(Result)
}
