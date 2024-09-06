use axum::{response::Html, body::Body, routing::{get, post}, Router, Json, http::{header, HeaderMap, StatusCode}, response::{
    Redirect,
    Response,
    IntoResponse,
}, Extension};
use axum_extra::extract::{Form};
use serde::Deserialize;
use std::{
    net::SocketAddr,
    sync::Arc,
};
use anyhow::{Result, anyhow};
use crate::jwt::{Jwt, JwtError};

const FRONTEND_DIR: &'static str = "../../frontend";

#[derive(Debug, Deserialize)]
struct LoginQuery {
    email:      Option<String>,
    password:   Option<String>,
}

pub struct Server {
}

struct JwtVerification;

fn is_valid_email(email: &str) -> bool {
    let re = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.(com|asia)$").unwrap();
    re.is_match(email)
}

impl Server {
    // pub async fn new() -> Self {
    //     let jwt = JwtGenerator::new("secret".to_string(), 86_400);
    //     Self { jwt }
    // }

    pub async fn start() -> Result<()> {
        let app =
            Router::new()
                .route("/popup.js", get(|| async { Html(include_str!("../../frontend/popup.js")) }))
                .route("/login",    get(|| async { Html(include_str!("../../frontend/login.html")) }))
                .route("/login",    post(Self::login))
                .route("/register", get(|| async { Html(include_str!("../../frontend/register.html")) }))
                .route("/register", post(Self::register))
                .route("/chat",     get(Self::chat))
                .route("/test",     get(Self::test));

        let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
        println!("listening on {}", listener.local_addr()?);
        axum::serve(listener, app).await?;

        Ok(())
    }

    async fn login(Form(query): Form<LoginQuery>) -> Html<&'static str> {
        let LoginQuery{email, password} = query;
        if email.is_some() && password.is_some() {
            if is_valid_email(email.as_ref().unwrap()) {
                if Self::check_login(LoginQuery{email, password}).await {
                    return Html(include_str!("../../frontend/chat.html"));
                }
            }
        }
        Html(include_str!("../../frontend/login.html"))
    }

    async fn check_login(param: LoginQuery) -> bool {
        true
    }

    async fn register() -> Html<&'static str> {
        Html(include_str!("../../frontend/register.html"))
    }
    async fn chat() -> Html<&'static str> {
        Html(include_str!("../../frontend/chat.html"))
    }


    async fn test(jwt: Jwt) -> Result<String, JwtError> {
        jwt.verify()?;
        Ok("Welcome to the protected area :)".to_string())
    }
}

#[tokio::test]
async fn test() {
    Server::start().await.unwrap();

}