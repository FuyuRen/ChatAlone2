use axum::{
    Json,
    Router,
    response::Html,
    routing::{get, post},
    http::{StatusCode},
    response::{
        // Redirect,
        // Response,
        IntoResponse,
    }
};
// use axum_extra::extract::{Form};
use serde::{Deserialize, Serialize};

use anyhow::{Result, anyhow};
use serde_json::{json};
use crate::jwt::{Jwt, JwtError};
use crate::uuid::UUID;

const FRONTEND_DIR: &'static str = "../../frontend";

#[derive(Debug, Deserialize)]
struct LoginQuery {
    email:      Option<String>,
    password:   Option<String>,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct ServerResponse<'a> {
//     status: &'a str,
//     error:  Option<&'a str>,
//     data:   Option<Value>,
// }
//
// impl IntoResponse for ServerResponse<'_> {
//     fn into_response(self) -> Response {
//
//     }
// }

pub struct Server {
}

struct JwtVerification;

fn is_valid_email(email: &str) -> bool {
    let re = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.(com|asia)$").unwrap();
    re.is_match(email)
}

pub async fn start() -> Result<()> {
    let app =
        Router::new()
            .route("/popup.js", get(|| async { Html(include_str!("../../frontend/popup.js")) }))
            .route("/login",    get(|| async { Html(include_str!("../../frontend/login.html")) }))
            .route("/login",    post(login))
            .route("/register", get(|| async { Html(include_str!("../../frontend/register.html")) }))
            .route("/register", post(register))
            .route("/chat",     get(chat))
            .route("/test",     get(test));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn login(Json(query): Json<LoginQuery>) -> impl IntoResponse {
    let mut ret
        = (StatusCode::OK, Json(json!({"status": "error", "error": "Internal server error"})));

    let LoginQuery{email, password} = query;

    if email.is_none() || password.is_none() {
        ret.1 = Json::from(json!({"status": "error", "error": "Null email or password"}));
        return ret
    }
    if !is_valid_email(email.as_ref().unwrap()) {
        ret.1 = Json::from(json!({"status": "error", "error": "Illegal email or password"}));
        return ret
    }

    if let Some(id) = check_login(LoginQuery{email, password}).await {
        let jwt = Jwt::generate(i64::from(&id) as usize, 60);
        if let Ok(jwt) = jwt {
            ret.1 = Json::from(json!({"status": "ok", "token": jwt}));
            return ret
        };
    } else {
        ret.1 = Json::from(json!({"status": "error", "error": "Invalid email or password"}));
        return ret
    }

    ret
}

async fn check_login(param: LoginQuery) -> Option<UUID> {
    Some(UUID::new())
}

async fn register() -> Html<&'static str> {
    Html(include_str!("../../frontend/register.html"))
}
async fn chat(jwt: Jwt) -> Result<Html<&'static str>, JwtError> {
    jwt.verify()?;
    Ok(Html(include_str!("../../frontend/chat.html")))
}


async fn test(jwt: Jwt) -> Result<String, JwtError> {
    jwt.verify()?;
    Ok("Welcome to the protected area :)".to_string())
}


#[tokio::test]
async fn test_server() {
    start().await.unwrap();
}