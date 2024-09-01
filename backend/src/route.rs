use axum::{
    response::Html,
    routing::get,
    Router,
    Json,
};
use axum_extra::extract::Form;
use serde::Deserialize;
use std::{
    net::SocketAddr,
    sync::Arc,
};
use anyhow::{Result, anyhow};

#[derive(Debug, Deserialize)]
struct LoginQuery {
    email:      Option<String>,
    password:   Option<String>,
}

pub struct Server;

fn is_valid_email(email: &str) -> bool {
    let re = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.com$").unwrap();
    re.is_match(email)
}

impl Server {
    pub async fn start() -> Result<()> {
        let app =
            Router::new()
                .route("/popup.js", get(||async { Html(include_str!("../../frontend/popup.js")) }))
                .route("/login",    get(Self::login))
                .route("/register", get(Self::register))
                .route("/chat",     get(Self::chat));

        // run it
        let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
            .await?;
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
}

#[tokio::test]
async fn test() {
    Server::start().await.unwrap();
}