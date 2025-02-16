use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::cookie;
use serde::Deserialize;

use crate::server::{
    fs_read, is_valid_email, AppState, ServerResponse, ServerResponseError, JWT_EXPIRE_DURATION,
};

use crate::jwt::Jwt;
use crate::sql::{
    DataBase,
    user,
};
use crate::id::{GeneralId, UserId};

/// req:
/// {
///     type: enum{ plain | markdown | image | file },
///     room_id:    u32,
///     content:    String,
///     quote_id:   Option<u32>,
/// }
/// ret:
/// {
///     msg_id:     u32;
///     timestamp:  u32;
/// }
/// 
///

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum RoomMessageParams {
    #[serde(rename = "1")]
    PlainText { room_id: u32, content: String, quote_id: Option<u32> },
    #[serde(rename = "2")]
    Markdown  { room_id: u32, content: String, quote_id: Option<u32> },
}

async fn post_message(
    jwt: Jwt,
    State(state): State<AppState>,
    Json(params): Json<RoomMessageParams>,
) -> impl IntoResponse {
    let user_id = UserId::from_encoded(jwt.user_id() as u32).decode();
    
}