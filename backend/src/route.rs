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
use serde::{Deserialize, Serialize};

use anyhow::{Result, anyhow, Error};
use axum::extract::State;
use axum::http::Response;
use serde_json::{json};
use tokio::fs::File;
use tokio::io;
use tokio::io::AsyncReadExt;
use crate::email::Email;
use crate::jwt::{Jwt, JwtError};
use crate::sql::{UserDB, UserTable};
use crate::uuid::UUID;

const FRONTEND_DIR: &'static str = "../../frontend";
const DISABLE_DYNAMIC_LOADING: bool = false;


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
        .route("/popup.js", get(get_popup))
        .route("/login",    get(get_login))
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
    let mut ret
        = (StatusCode::OK, Json(json!({"status": "error", "error": "Internal server error"})));

    println!("post(login) called with params: {:?}", params);
    let LoginParams{email, password} = params;

    if email.is_none() || password.is_none() {
        ret.1 = Json::from(json!({"status": "error", "error": "Null email or password"}));
        return ret
    }

    let email = email.unwrap();
    let password = password.unwrap();

    if ! is_valid_email(&email) {
        ret.1 = Json::from(json!({"status": "error", "error": "Illegal email or password"}));
        return ret
    }

    if let Some(id) = check_login(&state.user_db, &email, &password).await {
        println!("post(login) user found");
        let jwt = Jwt::generate(i64::from(&id) as usize, 60);
        if let Ok(jwt) = jwt {
            ret.1 = Json::from(json!({"status": "ok", "token": jwt}));
            return ret
        };
    } else {
        println!("post(login) user not found");
        ret.1 = Json::from(json!({"status": "error", "error": "Invalid email or password"}));
        return ret
    }

    ret
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
        return (StatusCode::OK, Json(json!({"status": "error", "error": "Invalid register params"})));
    }
    let user = db.select(params.email.as_ref().unwrap()).await;

    if let Ok(_) = user {
        return (StatusCode::OK, Json(json!({"status": "error", "error": "Email already registered"})));
    }
    if let Err(e) = &user {
        if !e.to_string().eq("user not found") {
            return (StatusCode::OK, Json(json!({"status": "error", "error": "Internal server error"})));
        }
    }
    drop(user);

    let new_user: UserTable = params.try_into().unwrap();
    if let Err(_) = db.insert(&new_user).await {
        (StatusCode::OK, Json(json!({"status": "error", "error": "Internal server error"})))
    } else {
        (StatusCode::OK, Json(json!({"status": "ok"})))
    }

}

async fn chat(jwt: Jwt) -> Result<Html<&'static str>, JwtError> {
    jwt.verify()?;
    Ok(Html(include_str!("../../frontend/chat.html")))
}


async fn test(jwt: Jwt) -> Result<String, JwtError> {
    jwt.verify()?;
    Ok("Welcome to the protected area :)".to_string())
}

async fn handler_404() -> Html<&'static str> {
    Html::from("<html><body><h1>404 Not Found :(</h1></body></html>")
}
