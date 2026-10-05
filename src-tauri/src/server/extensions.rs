#![allow(non_snake_case)]

use std::sync::Arc;
use axum::{extract::{Path, State}, http::{header, StatusCode}, response::{Html, IntoResponse, Response}, Json};

use crate::AppState;

pub async fn List(State(AppState): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(AppState.Extensions.List(&AppState.Db))
}

#[derive(serde::Deserialize)]
pub(crate) struct ToggleRequest {
    #[serde(rename = "Enabled")]
    enabled: bool,
}

pub async fn Toggle(
    State(AppState): State<Arc<AppState>>,
    Path(Id): Path<String>,
    Json(Req): Json<ToggleRequest>,
) -> StatusCode {
    let Registry = Arc::clone(&AppState);
    let Done = tokio::task::spawn_blocking(move || Registry.Extensions.SetEnabled(&Registry.Db, &Id, Req.enabled)).await;
    match Done {
        Ok(true) => StatusCode::OK,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn DisableAll(State(AppState): State<Arc<AppState>>) -> StatusCode {
    let Registry = Arc::clone(&AppState);
    match tokio::task::spawn_blocking(move || Registry.Extensions.DisableAll(&Registry.Db)).await {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn Reload(State(AppState): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let Registry = Arc::clone(&AppState);
    let _ = tokio::task::spawn_blocking(move || Registry.Extensions.Load(&Registry.Db)).await;
    Json(AppState.Extensions.List(&AppState.Db))
}

pub async fn PanelModule(State(AppState): State<Arc<AppState>>, Path(Id): Path<String>) -> Response {
    match AppState.Extensions.PanelSource(&AppState.Db, &Id) {
        Some(Source) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "text/javascript; charset=utf-8"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            Source,
        ).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

const SAFE_PAGE: &str = include_str!("safe.html");

pub async fn Safe() -> Response {
    (
        [
            (header::CONTENT_SECURITY_POLICY, "default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; connect-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'"),
            (header::CACHE_CONTROL, "no-store"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        Html(SAFE_PAGE),
    ).into_response()
}
