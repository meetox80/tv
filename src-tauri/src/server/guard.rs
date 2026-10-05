#![allow(non_snake_case)]

use std::sync::Arc;
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::AppState;
use crate::log::{Subsystem::Server, Level::*};

pub async fn RequireSession(
    State(AppState): State<Arc<AppState>>,
    Req: Request,
    Next: Next,
) -> Response {
    let Token = Req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|V| V.to_str().ok())
        .and_then(|V| V.strip_prefix("Bearer "))
        .unwrap_or("")
        .trim();

    if !AppState.ValidSession(Token) {
        log!(Server, Warn, "rejected unauthenticated {}", Req.uri().path());
        return StatusCode::UNAUTHORIZED.into_response();
    }

    Next.run(Req).await
}
