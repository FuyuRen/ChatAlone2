use anyhow::anyhow;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use crate::server::{fs_read, AppState, ServerResponse, ServerResponseError};
use crate::sql::{BasicCRUD, DataBase, UserDB, UserTable};

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
    type Error = anyhow::Error;
    fn try_into(self) -> Result<UserTable, Self::Error> {
        if self.is_legal() {
            Ok(UserTable::new(
                &self.email.unwrap(),
                &self.username.unwrap(),
                &self.password.unwrap(),
            ))
        } else {
            Err(anyhow!("Invalid register params"))
        }
    }
}

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
    let db = UserDB::from_state(&state);
    if !params.is_legal() {
        return ServerResponse::fine(ServerResponseError::InvalidRegisterParams, None);
    }
    let user = db.select_by_email(params.email.as_ref().unwrap()).await;

    if let Ok(res) = &user {
        if let Some(_) = res {
            return ServerResponse::fine(ServerResponseError::ExistRegisterEmail, None);
        }
    } else {
        println!("[Register(post)] Error: {}", user.err().unwrap());
        return ServerResponse::inner_err(ServerResponseError::InternalDatabaseError);
    }

    let new_user: UserTable = params.try_into().unwrap();
    if let Err(_) = db.insert(new_user).await {
        ServerResponse::inner_err(ServerResponseError::InternalDatabaseError)
    } else {
        ServerResponse::ok(None)
    }
}
