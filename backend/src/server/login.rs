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
use crate::sql::{DataBase, UserDB};
use crate::id::{GeneralId, UserId};

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

async fn check_login(user_db: &UserDB, email: &String, password: &String) -> Option<UserId> {
    let user = user_db.select_by_email(email).await;
    if let Err(_) = user {
        return None;
    }
    match user.unwrap() {
        Some(user) => user.verify_password(password).then(||{ user.uid() }),
        _ => None,
    }
}

async fn post_login(
    State(state): State<AppState>,
    Json(params): Json<LoginParams>
) -> impl IntoResponse {
    println!("post(login) called with params: {:?}", params);
    let LoginParams { email, password } = params;

    if email.is_none() || password.is_none() {
        return ServerResponse::fine(ServerResponseError::NullLoginParams, None);
    }

    let email = email.unwrap();
    let password = password.unwrap();

    if !is_valid_email(&email) {
        return ServerResponse::fine(ServerResponseError::IllegalLoginParams, None);
    }

    let db = UserDB::from_state(&state);
    if let Some(uid) = check_login(&db, &email, &password).await {
        println!("post(login) user found");
        let jwt = Jwt::generate(uid.encode() as usize, JWT_EXPIRE_DURATION);
        if let Ok(jwt) = jwt {
            let jwt_cookie = cookie::Cookie::build(("token", jwt))
                .path("/")
                .max_age(time::Duration::seconds(JWT_EXPIRE_DURATION))
                .http_only(true)
                .secure(false)
                .build();

            return ServerResponse::ok(None)
                .set_cookie(jwt_cookie)
                .unwrap_or_else(|_e| {
                    ServerResponse::inner_err(ServerResponseError::InternalTokenGenError)
                });
        };
    } else {
        println!("post(login) user not found");
        return ServerResponse::fine(ServerResponseError::InvalidLoginParams, None);
    }

    ServerResponse::inner_err(ServerResponseError::InternalUnknownError)
}
