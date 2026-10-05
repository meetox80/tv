#![allow(non_snake_case)]

use std::sync::Arc;
use std::collections::HashMap;
use axum::{extract::{Query, State}, http::StatusCode, Json};

use crate::AppState;
use crate::db::{ScreenPage, PAGE_SECONDS};
use crate::log::{Subsystem::Server, Level::*};

pub async fn GetScreen(State(AppState): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(AppState.ScreenConfig())
}

pub async fn GetPreview(State(AppState): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let Track = AppState.LastTrack.lock().unwrap().clone();
    let Showing = AppState.Showing.lock().unwrap().as_ref().map(|S| S.Pages.clone());
    let Pages = Showing.unwrap_or_else(|| {
        AppState.Db.GetScreenPages().into_iter().filter(|P| P.Enabled).map(|P| P.Id).collect()
    });

    Json(serde_json::json!({
        "Track": Track,
        "Cover": AppState.CoverPath(),
        "Notice": AppState.Db.GetScreenNotice(),
        "Weather": {
            "Enabled": AppState.Db.GetWeatherEnabled(),
            "Latitude": AppState.Db.GetWeatherLatitude(),
            "Longitude": AppState.Db.GetWeatherLongitude(),
        },
        "Pages": Pages,
    }))
}

pub async fn GetLive(State(AppState): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let Held = AppState.Showing.lock().unwrap();
    let Live = Held.as_ref().and_then(|S| {
        let Elapsed = S.Since.elapsed().as_millis() as u64;
        let Stale = Elapsed > S.Seconds as u64 * 1000 + 5000;
        (!Stale).then(|| serde_json::json!({
            "Page": S.Page,
            "Pages": S.Pages,
            "Seconds": S.Seconds,
            "Elapsed": Elapsed,
        }))
    });
    Json(Live.unwrap_or(serde_json::Value::Null))
}

#[derive(serde::Deserialize)]
pub(crate) struct NoticeRequest {
    #[serde(rename = "Notice")]
    notice: String,
}

pub async fn SaveNotice(
    State(AppState): State<Arc<AppState>>,
    Json(Req): Json<NoticeRequest>,
) -> StatusCode {
    let Text = Req.notice.trim();

    if Text.chars().count() > 300 {
        return StatusCode::BAD_REQUEST;
    }

    AppState.Db.SetScreenNotice(Text);
    AppState.NotifyScreen();

    log!(Server, Info, "notice set ({} chars)", Text.chars().count());
    StatusCode::OK
}

pub async fn GetLessons(
    State(AppState): State<Arc<AppState>>,
    Query(Params): Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let Duration = Params
        .get("duration")
        .and_then(|D| D.parse::<u8>().ok())
        .filter(|D| *D == 30 || *D == 45)
        .unwrap_or_else(|| AppState.Db.GetLessonDuration());

    Json(crate::lessons::AsJson(Duration))
}

#[derive(serde::Deserialize)]
pub(crate) struct PagesRequest {
    #[serde(rename = "Pages")]
    pages: Vec<ScreenPage>,
}

pub async fn SavePages(
    State(AppState): State<Arc<AppState>>,
    Json(Req): Json<PagesRequest>,
) -> StatusCode {
    let Known = AppState.Db.ScreenPageIds();
    let Complete = Req.pages.len() == Known.len()
        && Known.iter().all(|(Id, _)| Req.pages.iter().filter(|P| P.Id == *Id).count() == 1);
    if !Complete || Req.pages.iter().any(|P| !PAGE_SECONDS.contains(&P.Seconds)) {
        return StatusCode::BAD_REQUEST;
    }

    AppState.Db.SetScreenPages(&Req.pages);
    AppState.NotifyScreen();

    let Summary: Vec<String> = Req.pages
        .iter()
        .map(|P| format!("{}{} {}s", if P.Enabled { "" } else { "-" }, P.Id, P.Seconds))
        .collect();
    log!(Server, Info, "screen pages set: {}", Summary.join(", "));
    StatusCode::OK
}

#[derive(serde::Deserialize)]
pub(crate) struct LessonsRequest {
    #[serde(rename = "Duration")]
    duration: u8,
}

pub async fn SaveLessons(
    State(AppState): State<Arc<AppState>>,
    Json(Req): Json<LessonsRequest>,
) -> StatusCode {
    if Req.duration != 30 && Req.duration != 45 {
        return StatusCode::BAD_REQUEST;
    }

    AppState.Db.SetLessonDuration(Req.duration);
    AppState.NotifyScreen();

    log!(Server, Info, "lesson duration set to {}", Req.duration);
    StatusCode::OK
}

#[derive(serde::Deserialize)]
pub(crate) struct WeatherRequest {
    #[serde(rename = "Enabled")]
    enabled: bool,
    #[serde(rename = "Latitude")]
    latitude: f64,
    #[serde(rename = "Longitude")]
    longitude: f64,
}

pub async fn SaveWeather(
    State(AppState): State<Arc<AppState>>,
    Json(Req): Json<WeatherRequest>,
) -> StatusCode {
    if !(-90.0..=90.0).contains(&Req.latitude) || !(-180.0..=180.0).contains(&Req.longitude) {
        return StatusCode::BAD_REQUEST;
    }

    AppState.Db.SetWeather(Req.enabled, Req.latitude, Req.longitude);
    AppState.NotifyScreen();

    log!(Server, Info, "weather set to {}, {}", Req.latitude, Req.longitude);
    StatusCode::OK
}
