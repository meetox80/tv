#![allow(non_snake_case)]

use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::Response,
};

use crate::AppState;
use crate::log::{Subsystem::Server, Level::*};

const HANDSHAKE_SECS: u64 = 5;

pub async fn PerformanceSocket(
    State(AppState): State<Arc<AppState>>,
    Upgrade: WebSocketUpgrade,
) -> Response {
    Upgrade.on_upgrade(move |Socket| _Stream(AppState, Socket))
}

async fn _Token(Socket: &mut WebSocket) -> Option<String> {
    let First = tokio::time::timeout(Duration::from_secs(HANDSHAKE_SECS), Socket.recv())
        .await
        .ok()??;

    match First.ok()? {
        Message::Text(Text) => Some(Text.as_str().trim().to_string()),
        _ => None,
    }
}

async fn _Send(AppState: &Arc<AppState>, Socket: &mut WebSocket) -> bool {
    let Payload = crate::perf::Snapshot(AppState).to_string();
    Socket.send(Message::Text(Payload.into())).await.is_ok()
}

async fn _Stream(AppState: Arc<AppState>, mut Socket: WebSocket) {
    let Token = match _Token(&mut Socket).await {
        Some(T) if AppState.ValidSession(&T) => T,
        _ => {
            log!(Server, Warn, "rejected unauthenticated performance socket");
            let _ = Socket.send(Message::Close(None)).await;
            return;
        }
    };

    let mut Ticks = AppState.PerfTick.subscribe();

    if !_Send(&AppState, &mut Socket).await {
        return;
    }

    loop {
        if Ticks.changed().await.is_err() {
            return;
        }

        if !AppState.ValidSession(&Token) {
            let _ = Socket.send(Message::Close(None)).await;
            return;
        }

        if !_Send(&AppState, &mut Socket).await {
            return;
        }
    }
}
