use axum::{
    Router,
    routing::get,
    response::IntoResponse,
    extract::{
        ws::{
            Message,
            WebSocket,
            WebSocketUpgrade,
        },
        ConnectInfo
    }
};
use axum_extra::TypedHeader;
use axum_extra::extract::Form;

use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio::sync::mpsc;
use tokio::net::unix::SocketAddr;
use futures::{SinkExt, StreamExt};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::sync::Arc;
use std::time::Duration;
use axum::extract::State;
use dashmap::DashMap;
use crate::jwt::{Jwt, JwtError};
use crate::server::{AppState, ServerResponse};
use crate::sql::UserDB;
use crate::uuid::UUID;

struct WsAppState {
    session_pool: DashMap<UUID, WsSession>
}

impl Clone for WsAppState {
    fn clone(&self) -> Self {
        Self {
            session_pool: self.session_pool.clone()
        }
    }
}

pub(crate) fn route(app_state: AppState) -> Router {
    Router::new()
        .get("/ws", get(handler))
        .with_state(app_state)
}

#[derive(Debug, Deserialize)]
struct WsParam {
    token: Option<String>,
    sync: Option<u64>,
    session_id: Option<UUID>
}

impl WsParam {
    fn parse_valid(&self) -> WsError {
        let token = &self.token;
        if let Some(token) = token {
            match Jwt::parse_and_verify(token) {
                Ok(_) => WsError::Success,
                Err(e) => {
                    match e {
                        JwtError::InvalidToken      => WsError::InvalidToken,
                        JwtError::Expired           => WsError::TokenExpired,
                        JwtError::InternalError(_)  => WsError::InternalError,
                    }
                }
            }
        } else {
            WsError::MissingParam
        }
    }

    fn is_resume(&self) -> bool {
        true
    }
}

async fn handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Form(param): Form<WsParam>,
    State(app_state): State<AppState>,
    State(ws_session): State<WsAppState>
) -> impl IntoResponse {
    let res = param.parse_valid();
    if !param.parse_valid().is_ok() { return res }

    ws.on_upgrade(move |socket| ws_handler(socket, addr))
}

#[derive(Clone)]
enum WsError {
    Success,
    MissingParam,
    InvalidToken,
    IllegalToken,
    TokenExpired,
    InternalError
}

impl WsError {
    fn is_ok(&self) -> bool {
        self.clone() as u32 == 0
    }
}

enum WsSignal {
    Ping,
    ResumeOK,
    Message(WsMessage),
    ConnectOK(Value),
}

#[derive(Serialize)]
struct WsMessage {
    data: Value,
    sync: u64
}

impl WsMessage {
    fn new(data: Value, sync: u64) -> Self {
        Self { data, sync }
    }
}

struct WsSession {
    db_handler:     Arc<Mutex<UserDB>>,
    sync_base:      u64,
    sync_offset:    u64,
}

async fn ws_handler(mut socket: WebSocket, addr: SocketAddr) -> anyhow::Result<()> {
    let (mut sender, mut receiver) = socket.split();

    // Auth
    sender.send(Message::Text("[AUTH] Token?".into())).await?;

    tokio::select!{
        Some(Ok(Message::Text(token))) = receiver.next().await => {
            let token = token.trim_start_matches("Bearer ");
            if let Err(e) = Jwt::parse_and_verify(token) {
                return Err(e.into())
            }、
        },
        _ = sleep(Duration::from_secs(3)) => {
            return Err(anyhow::anyhow!("timeout"));
        }
    }

    loop {

    }
}