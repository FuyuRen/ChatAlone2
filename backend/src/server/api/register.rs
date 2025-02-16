use std::sync::OnceLock;
use anyhow::anyhow;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use dashmap::DashMap;
use sea_orm::ActiveValue;
use serde::Deserialize;
use crate::server::{fs_read, AppState, ServerResponse, ServerResponseError};
use crate::sql::{
    BasicCRUD,
    DataBase,
    user,
};
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

impl TryInto<user::ActiveModel> for RegisterParams {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<user::ActiveModel, Self::Error> {
        if self.is_legal() {
            Ok(user::ActiveModel {
                email:      ActiveValue::Set(self.email.unwrap()),
                username:   ActiveValue::Set(self.username.unwrap()),
                password:   ActiveValue::Set(self.password.unwrap()),
                ..Default::default()
            })
        } else {
            Err(anyhow!("Invalid register params"))
        }
    }
}

// struct RegisterSession(DashMap<u32, RegisterParams>);
// 
// static REGISTER_SESSION: OnceLock<RegisterSession> = RegisterSession(DashMap::new());

pub(crate) fn route(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/register", get(get_register))
        .route("/register", post(post_register))
        .with_state(app_state)
}

async fn get_register() -> Html<String> {
    Html(fs_read("../frontend/register.html").await.unwrap())
}

async fn post_register(
    State(state): State<AppState>,
    Json(params): Json<RegisterParams>,
) -> impl IntoResponse {
    println!("post(register) called with params: {:?}", params);
    let user_db = user::DB::from_state(&state);
    if !params.is_legal() {
        return ServerResponse::fine(ServerResponseError::InvalidRegisterParams, None);
    }
    let user = user_db.select_email(params.email.as_ref().unwrap()).await;

    if let Ok(res) = &user {
        if let Some(_) = res {
            return ServerResponse::fine(ServerResponseError::ExistRegisterEmail, None);
        }
    } else {
        println!("[Register(post)] Error: {}", user.err().unwrap());
        return ServerResponse::inner_err(ServerResponseError::InternalDatabaseError);
    }

    let new_user: user::ActiveModel = params.try_into().unwrap();
    if let Err(_) = user_db.insert(new_user).await {
        ServerResponse::inner_err(ServerResponseError::InternalDatabaseError)
    } else {
        ServerResponse::ok(None)
    }
}
