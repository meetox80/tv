#![allow(non_snake_case)]

use std::sync::Arc;
use axum::{body::Bytes, extract::{Path, State}, http::{header, StatusCode}, response::{IntoResponse, Response}, Json};
use serde_json::json;

use crate::AppState;
use crate::slides::{self, Slide};
use crate::log::{Subsystem::Server, Level::*};

fn _Refuse(Status: StatusCode, Message: &str) -> Response {
    (Status, Json(json!({ "Error": Message }))).into_response()
}

fn _Changed(AppState: &AppState) {
    AppState.NotifySlides();
    AppState.NotifyScreen();
}

pub async fn List(State(AppState): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(slides::List(&AppState.Db))
}

pub async fn Get(State(AppState): State<Arc<AppState>>, Path(Id): Path<String>) -> Response {
    match slides::Get(&AppState.Db, &Id) {
        Some(Collection) => Json(Collection).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct CreateRequest {
    #[serde(rename = "Name")]
    name: String,
}

pub async fn Create(State(AppState): State<Arc<AppState>>, Json(Req): Json<CreateRequest>) -> Response {
    match slides::Create(&AppState.Db, &Req.name) {
        Ok(Collection) => {
            log!(Server, Info, "slide collection created");
            _Changed(&AppState);
            (StatusCode::CREATED, Json(Collection)).into_response()
        }
        Err(E) => _Refuse(StatusCode::BAD_REQUEST, &E),
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct SaveRequest {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Slides")]
    slides: Vec<Slide>,
}

pub async fn Save(State(AppState): State<Arc<AppState>>, Path(Id): Path<String>, Json(Req): Json<SaveRequest>) -> Response {
    match slides::Save(&AppState.Db, &Id, &Req.name, Req.slides) {
        Ok(Updated) => {
            _Changed(&AppState);
            Json(json!({ "Updated": Updated })).into_response()
        }
        Err(E) => _Refuse(StatusCode::BAD_REQUEST, &E),
    }
}

pub async fn Delete(State(AppState): State<Arc<AppState>>, Path(Id): Path<String>) -> StatusCode {
    if slides::Delete(&AppState.Db, &Id) {
        log!(Server, Info, "slide collection deleted");
        _Changed(&AppState);
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn Upload(State(AppState): State<Arc<AppState>>, Body: Bytes) -> Response {
    match slides::PutAsset(&AppState.Db, &Body) {
        Ok(Id) => (StatusCode::CREATED, Json(json!({ "Id": Id }))).into_response(),
        Err(E) => _Refuse(StatusCode::BAD_REQUEST, &E),
    }
}

pub async fn Asset(State(AppState): State<Arc<AppState>>, Path(Id): Path<String>) -> Response {
    match slides::Asset(&AppState.Db, &Id) {
        Some((Bytes, Mime)) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, Mime),
                (header::CACHE_CONTROL, "public, max-age=31536000, immutable".to_string()),
            ],
            Bytes,
        ).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
