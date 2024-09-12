use std::fmt::Display;

use tokio::fs::File;
use tokio::io;
use tokio::io::AsyncReadExt;
use anyhow::{Result, anyhow, Error};

use serde::{ ser::SerializeMap, Deserialize, Serialize };
use serde_json::{json, Value};

use axum::{
    Json,
    Router,
    body::Body,
    response::Html,
    routing::{get, post},
    extract::State,
    http::{
        header,
        HeaderName,
        StatusCode,
    },
    response::{
        Redirect,
        Response,
        IntoResponse,
    }
};
use axum_extra::{
    headers::{Cookie, HeaderMap},
    extract::cookie,
};

use crate::email::Email;
use crate::jwt::{Jwt, JwtError};
use crate::sql::{UserDB, UserTable};
use crate::uuid::UUID;

const FRONTEND_DIR: &'static str = "../../frontend";
const DISABLE_DYNAMIC_LOADING: bool = false;
const JWT_EXPIRE_DURATION: i64 = 30;


#[derive(Debug, Deserialize)]
struct LoginParams {
    email:      Option<String>,
    password:   Option<String>,
}

#[derive(Debug, Deserialize)]
struct RegisterParams {
    email:          Option<String>,
    username:       Option<String>,
    password:       Option<String>,
}

impl RegisterParams {
    pub fn is_legal(&self) -> bool {
        self.username.is_some() && self.password.is_some() && self.email.is_some()
    }
}

impl TryInto<UserTable> for RegisterParams {
    type Error = Error;
    fn try_into(self) -> std::result::Result<UserTable, Self::Error> {
        if self.is_legal() {
            Ok(UserTable::new(
                &self.email.unwrap(),
                &self.username.unwrap(),
                &self.password.unwrap()
            ))
        } else {
            Err(anyhow!("Invalid register params"))
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum ServerResponseError {
    SUCCESS,
    InvalidRegisterParams,
    InternalErrorError,
}
impl ServerResponseError {
    fn code(&self) -> u32 {
        *self as u32
    }
    fn message(&self) -> &'static str {
        match self {
            ServerResponseError::SUCCESS => "Success",
            ServerResponseError::InvalidRegisterParams => "Invalid register params",
            ServerResponseError::InternalErrorError => "Internal error",
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

    fn fine( error: ServerResponseError, data: Option<Value>) -> Self {
        Self::new(StatusCode::OK, error, data)
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

    fn append_header(mut self, (k, v): (HeaderName, &str)) -> Result<Self> {
        self.headers.append(k, v.parse()?);
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
    where S: serde::Serializer {
        let mut map = if self.error.is_success() {
            serializer.serialize_map(Some(2))?
        } else {
            let mut map = serializer.serialize_map(Some(3))?;
            map.serialize_entry("errmsg", self.error.message())?;
            map
        };
        map.serialize_entry("errcode", &(self.error.code()))?;
        map.serialize_entry("data", &self.data)?;
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
    user_db: UserDB,
}
impl AppState {
    pub fn new(user_db: UserDB) -> Self {
        Self { user_db }
    }
}

struct JwtVerification;

fn is_valid_email(email: &str) -> bool {
    let re = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.(com|asia)$").unwrap();
    re.is_match(email)
}

pub async fn new(addr: &str, user_db: UserDB) -> Result<()> {
    let state = AppState::new(user_db);

    let app = Router::new()
        .route("/login",    get(get_login))
        .route("/popup.js", get(get_popup))
        .route("/login",    post(post_login))
        .route("/register", get(get_register))
        .route("/register", post(post_register))
        .route("/chat",     get(chat))
        .route("/test",     get(test))
        .with_state(state);

    let app = app.fallback(handler_404);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_popup() -> Html<String> {
    if DISABLE_DYNAMIC_LOADING {
        Html(include_str!("../../frontend/popup.js").parse().unwrap())
    } else {
        Html(fs_read("../frontend/popup.js").await.unwrap())
    }
}
async fn get_login() -> Html<String> {
    if DISABLE_DYNAMIC_LOADING {
        Html(include_str!("../../frontend/login.html").parse().unwrap())
    } else {
        Html(fs_read("../frontend/login.html").await.unwrap())
    }
}
async fn get_register() -> Html<String> {
    if DISABLE_DYNAMIC_LOADING {
        Html(include_str!("../../frontend/register.html").parse().unwrap())
    } else {
        Html(fs_read("../frontend/register.html").await.unwrap())
    }
}

async fn post_login(state: State<AppState>, Json(params): Json<LoginParams>) -> impl IntoResponse {
    let ret = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json");

    // let mut ret = (
    //     StatusCode::OK,
    //     Json(json!({"status": "error", "error": "Internal unknown error"}))
    // );

    println!("post(login) called with params: {:?}", params);
    let LoginParams{email, password} = params;

    if email.is_none() || password.is_none() {
        // return ret.body(Body::from(json!({"status": "error", "error": "Null email or password"}).to_string())).unwrap();
        return ServerResponse::fine(
            ServerResponseError::InternalErrorError,
            Some(json!({"status": "error", "error": "Null email or password"}))
        );
    }

    let email = email.unwrap();
    let password = password.unwrap();

    if ! is_valid_email(&email) {
        return ret.body(Body::from(json!({"status": "error", "error": "Illegal email or password"}).to_string())).unwrap();
    }

    if let Some(id) = check_login(&state.user_db, &email, &password).await {
        println!("post(login) user found");
        let jwt = Jwt::generate(i64::from(&id) as usize, JWT_EXPIRE_DURATION);
        if let Ok(jwt) = jwt {
            let jwt_cookie = cookie::Cookie::build(("token", jwt))
                .path("/")
                .max_age(time::Duration::hours(1))
                .http_only(true)
                .secure(false)
                .build();

            return ret.header(header::SET_COOKIE, jwt_cookie.to_string())
                        .body(Body::from(json!({"status": "ok"}).to_string())).unwrap();
        };
    } else {
        println!("post(login) user not found");
        return ret.body(Body::from(json!({"status": "error", "error": "Invalid email or password"}).to_string())).unwrap();
    }

    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header("Content-Type", "application/json")
        .body(Body::from(json!({"status": "error", "error": "Internal unknown error"}).to_string())).unwrap()
}

async fn check_login(user_db: &UserDB, email: &String, password: &String) -> Option<UUID> {
    let user = user_db.select(email).await;
    if let Err(_) = user { return None }
    let user = user.unwrap();
    if !user.verify_password(password) { return None }
    Some(user.userid)
}

async fn post_register(state: State<AppState>, Json(params): Json<RegisterParams>) -> impl IntoResponse {
    let db = &state.user_db;
    if !params.is_legal() {
        return (
            StatusCode::OK,
            Json(json!({"status": "error", "error": "Invalid register params"}))
        );
    }
    let user = db.select(params.email.as_ref().unwrap()).await;

    if let Ok(_) = user {
        return (
            StatusCode::OK,
            Json(json!({"status": "error", "error": "Email already registered"}))
        );
    }
    if let Err(e) = &user {
        if !e.to_string().eq("user not found") {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error", "error": "Internal database error"}))
            );
        }
    }
    drop(user);

    let new_user: UserTable = params.try_into().unwrap();
    if let Err(_) = db.insert(&new_user).await {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error", "error": "Internal database error"}))
        )
    } else {
        (
            StatusCode::OK,
            Json(json!({"status": "ok"}))
        )
    }

}

async fn chat(_jwt: Jwt) -> Result<Html<String>, JwtError> {
    Ok(Html(include_str!("../../frontend/chat.html").to_string()))
}


async fn test(_jwt: Jwt) -> Result<String, JwtError> {
    Ok("Welcome to the protected area :)".to_string())
}

async fn handler_404() -> Html<&'static str> {
    Html::from("<html><body><h1>404 Not Found :(</h1></body></html>")
}
