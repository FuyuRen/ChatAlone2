use axum::{
    Json,
    Router,
    extract::State,
    http::StatusCode,
    response::{
        Html,
        Response,
        IntoResponse,
    },
    routing::{get, post},
};
use axum_extra::extract::cookie;
use serde::Deserialize;

use crate::server::{
    fs_read,
    is_valid_email,
    AppState,
    ServerResponse,
    ServerResponseError,
    JWT_EXPIRE_DURATION
};

use crate::jwt::Jwt;
use crate::sql::UserDB;
use crate::uuid::UUID;

#[derive(Debug, Deserialize)]
struct LoginParams {
    email:      Option<String>,
    password:   Option<String>,
}

pub(crate) fn route(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/login", get(get_login))
        .route("/login", post(post_login))
        .with_state(app_state)
}

async fn get_login() -> Html<String> {
    Html(fs_read("../frontend/login.html").await.unwrap())
}

async fn check_login(user_db: &UserDB, email: &String, password: &String) -> Option<UUID> {
    let user = user_db.select(email).await;
    if let Err(_) = user { return None }
    let user = user.unwrap();
    if !user.verify_password(password) { return None }
    Some(user.userid)
}

async fn post_login(state: State<AppState>, Json(params): Json<LoginParams>) -> impl IntoResponse {
    let ret = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json");

    println!("post(login) called with params: {:?}", params);
    let LoginParams{email, password} = params;

    if email.is_none() || password.is_none() {
        return ServerResponse::fine(ServerResponseError::NullLoginParams, None);
    }

    let email = email.unwrap();
    let password = password.unwrap();

    if ! is_valid_email(&email) {
        return ServerResponse::fine(ServerResponseError::IllegalLoginParams, None);
    }

    if let Some(id) = check_login(&state.user_db, &email, &password).await {
        println!("post(login) user found");
        let jwt = Jwt::generate(i64::from(&id) as usize, JWT_EXPIRE_DURATION);
        if let Ok(jwt) = jwt {
            let jwt_cookie = cookie::Cookie::build(("token", jwt))
                .path("/")
                .max_age(time::Duration::seconds(JWT_EXPIRE_DURATION))
                .http_only(true)
                .secure(false)
                .build();

            return ServerResponse::ok(None).set_cookie(jwt_cookie)
                .unwrap_or_else(|_e|
                    ServerResponse::inner_err(ServerResponseError::InternalTokenGenError)
                )
        };
    } else {
        println!("post(login) user not found");
        return ServerResponse::fine(ServerResponseError::InvalidLoginParams, None);
    }

    ServerResponse::inner_err(ServerResponseError::InternalUnknownError)
}
