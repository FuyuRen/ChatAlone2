mod login;
mod public;
mod register;
mod tools;
mod ws;

use std::fmt::Display;

use anyhow::{anyhow, Error, Result};
use tokio::fs::File;
use tokio::io;
use tokio::io::AsyncReadExt;

use serde::{ser::SerializeMap, Deserialize, Serialize};
use serde_json::{json, Value};

use axum::http::HeaderValue;
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderName, StatusCode},
    response::Html,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Json, Router,
};
use axum_extra::{
    extract::cookie,
    headers::{Cookie, HeaderMap},
};
use sea_orm::DatabaseConnection;
use crate::email::Email;
use crate::jwt::{Jwt, JwtError};
use crate::sql::{DataBase, UserDB};
use crate::entities::user_info::UserTable;

const FRONTEND_DIR: &'static str = "../../frontend";
const JWT_EXPIRE_DURATION: i64 = 3600;

#[derive(Debug, Copy, Clone)]
enum ServerResponseError {
    SUCCESS,
    NullLoginParams,
    IllegalLoginParams,
    InvalidLoginParams,

    InvalidRegisterParams,
    ExistRegisterEmail,

    InternalTokenGenError,
    InternalDatabaseError,
    InternalUnknownError,
}
impl ServerResponseError {
    fn code(&self) -> u32 {
        *self as u32
    }
    fn message(&self) -> &'static str {
        match self {
            ServerResponseError::SUCCESS => "Success",
            // --------------------------------login-------------------------------- //
            ServerResponseError::   NullLoginParams     =>    "Null email or password",
            ServerResponseError::IllegalLoginParams     => "Illegal email or password",
            ServerResponseError::InvalidLoginParams     => "Invalid email or password",
            // -------------------------------register------------------------------ //
            ServerResponseError::InvalidRegisterParams  =>   "Invalid register params",
            ServerResponseError::ExistRegisterEmail     =>      "Email already exists",
            // -----------------------------general-error--------------------------- //
            ServerResponseError::InternalTokenGenError  =>            "Internal error",
            ServerResponseError::InternalDatabaseError  =>            "Internal error",
            ServerResponseError::InternalUnknownError   =>    "Internal unknown error",
        }
    }

    fn is_success(&self) -> bool {
        self.code() == 0
    }
}
impl Display for ServerResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = self.message();
        write!(f, "{}", ret)?;
        Ok(())
    }
}

struct ServerResponse {
    status:     StatusCode,
    error:      ServerResponseError,
    headers:    HeaderMap,
    data:       Option<Value>
}

impl ServerResponse {
    fn ok(data: Option<Value>) -> Self {
        Self::new(StatusCode::OK, ServerResponseError::SUCCESS, data)
    }

    fn fine(error: ServerResponseError, data: Option<Value>) -> Self {
        Self::new(StatusCode::OK, error, data)
    }

    fn inner_err(error: ServerResponseError) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, error, None)
    }

    fn new(status: StatusCode, error: ServerResponseError, data: Option<Value>) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
        headers.append(header::ORIGIN, "*".parse().unwrap());

        ServerResponse {
            status,
            error,
            headers,
            data,
        }
    }

    fn has_header(&self, k: HeaderName) -> bool {
        self.headers.contains_key(k)
    }

    fn set_header(mut self, k: HeaderName, v: &str) -> Result<Self> {
        self.headers.insert(k, v.parse()?);
        Ok(self)
    }

    fn set_cookie(mut self, cookie: cookie::Cookie) -> Result<Self> {
        self.headers
            .insert(header::SET_COOKIE, cookie.to_string().parse()?);
        Ok(self)
    }

    fn set_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    fn data(mut self, data: Option<Value>) -> Self {
        self.data = data;
        self
    }
}

impl Default for ServerResponse {
    fn default() -> Self {
        Self::ok(None)
    }
}

impl Serialize for ServerResponse {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let len = 2;
        let mut map = if self.data.is_some() {
            let mut map = serializer.serialize_map(Some(len + 1))?;
            map.serialize_entry("data", &self.data)?;
            map
        } else {
            serializer.serialize_map(Some(len))?
        };

        map.serialize_entry("errmsg", self.error.message())?;
        map.serialize_entry("errcode", &(self.error.code()))?;
        map.end()
    }
}

impl IntoResponse for ServerResponse {
    fn into_response(self) -> Response {
        let value = json!(&self);
        let status = self.status;
        let headers = self.headers;

        (status, headers, Json::from(value)).into_response()
    }
}

pub async fn fs_read(path: &str) -> Result<String> {
    let file = File::open(path).await?;
    let mut reader = io::BufReader::new(file);

    let mut email_cfg = String::new();
    reader.read_to_string(&mut email_cfg).await?;
    Ok(email_cfg)
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub db_conn: DatabaseConnection,
}

impl AppState {
    pub fn new(db_conn: DatabaseConnection) -> Self {
        Self { db_conn }
    }
}

fn is_valid_email(email: &str) -> bool {
    regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.(com|asia)$").unwrap().is_match(email)
}

pub fn route(db_conn: DatabaseConnection) -> Router {
    let state = AppState::new(db_conn);

    let login = login::route(state.clone());
    let public = public::route(state.clone());
    let register = register::route(state.clone());
    let websocket = ws::route(state.clone());
    let tools = tools::route(state.clone());

    if cfg!(debug_assertions) {
        Router::new()
            .nest("/", login)
            .nest("/", register)
            .nest("/", public)
            .nest("/", websocket)
            .nest("/tools", tools)
            .route("/chat", get(chat))
            .fallback(handler_404)
            .with_state(state)
    } else {
        Router::new()
            .nest("/", login)
            .nest("/", register)
            .nest("/", public)
            .nest("/", websocket)
            .route("/chat", get(chat))
            .fallback(handler_404)
            .with_state(state)
    }
}

async fn chat(_jwt: Jwt) -> Html<String> {
    // Ok(Html(
    //     include_str!("../../../frontend/chat.html").to_string(),
    // ))
    Html(fs_read("../frontend/chat.html").await.unwrap())
}

async fn handler_404() -> Html<String> {
    Html::from("404 Not Found :( ".to_string())
}

// async fn handler_404() -> Html<&'static str> {
//     Html::from("<html><body><h1>404 Not Found :(</h1></body></html>")
// }
