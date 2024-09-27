use axum::{
    Json,
    Router,
    extract::State,
    response::{
        Html,
        IntoResponse,
    },
    routing::{get, post},
};
use anyhow::anyhow;
use serde::Deserialize;

use crate::sql::UserTable;
use crate::server::{
    fs_read,
    AppState,
    ServerResponse,
    ServerResponseError
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

impl TryInto<UserTable> for RegisterParams {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<UserTable, Self::Error> {
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

pub(crate) fn route(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/register", get(get_register))
        .route("/register", post(post_register))
        .with_state(app_state)
}

async fn get_register() -> Html<String> {
    Html(fs_read("../frontend/register.html").await.unwrap())
}

async fn post_register(state: State<AppState>, Json(params): Json<RegisterParams>) -> impl IntoResponse {
    let db = &state.user_db;
    if !params.is_legal() {
        return ServerResponse::fine(ServerResponseError::InvalidRegisterParams, None);
    }
    let user = db.select(params.email.as_ref().unwrap()).await;

    if let Err(e) = &user {
        if !e.to_string().eq("user not found") {
            return ServerResponse::inner_err(ServerResponseError::InternalDatabaseError);
        }
    } else {
        return ServerResponse::fine(ServerResponseError::ExistRegisterEmail, None);
    }

    let new_user: UserTable = params.try_into().unwrap();
    if let Err(_) = db.insert(&new_user).await {
        ServerResponse::inner_err(ServerResponseError::InternalDatabaseError)
    } else {
        ServerResponse::ok(None)
        // ServerResponse::new(StatusCode::PERMANENT_REDIRECT, ServerResponseError::SUCCESS, None)
    }

}