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

    let db = user::DB::from_state(&state);
    let user_model = match db.select_email(&email).await {
        Ok(Some(user_model)) => user_model,
        Ok(None) => {
            return ServerResponse::fine(ServerResponseError::InvalidLoginParams, None);
        },
        Err(_) => {
            return ServerResponse::inner_err(ServerResponseError::InternalDatabaseError);
        },
    };
    if user_model.password.eq(&password) {
        let user_id = UserId::from_decoded(user_model.id as u32);
        println!("post(login) user found");
        let jwt = Jwt::generate(user_id.encode() as usize, JWT_EXPIRE_DURATION);
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
