use crate::server::{fs_read, AppState};

use axum::{
    Router,
    routing::get
};
use axum::response::Html;

pub(crate) fn route(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/popup.js", get(popup))
        .route("/verify", get(get_verify))
        .with_state(app_state)
}

async fn popup() -> Html<String> {
    Html(fs_read("../frontend/popup.js").await.unwrap())
}

async fn get_verify() -> Html<String> {
    Html(fs_read("../frontend/verify.html").await.unwrap())
}