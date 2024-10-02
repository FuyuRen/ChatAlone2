use crate::jwt::Jwt;
use crate::server::AppState;
use crate::uuid::UUID;
use axum::extract::Query;
use axum::routing::get;
use axum::Router;
use serde::Deserialize;

pub(crate) fn route(state: AppState) -> Router<AppState> {
    Router::new().route("/jwt", get(new_jwt)).with_state(state)
}

#[derive(Debug, Deserialize)]
struct GenJwtQuery {
    id:     Option<usize>,
    expr:   Option<i64>,
}

async fn new_jwt(Query(query): Query<GenJwtQuery>) -> String {
    let expr = query.expr.unwrap_or(3600);
    let id = query.id.unwrap_or(UUID::new().into());
    Jwt::generate(id, expr).unwrap()
}
