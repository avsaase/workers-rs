use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::get, Router};
// use axum_macros::debug_handler;
use tower_service::Service;
use worker::*;

#[derive(Clone)]
pub struct AppState {
    env: Arc<Env>,
}

fn router(env: Env) -> Router {
    let state = AppState { env: Arc::new(env) };
    Router::new().route("/", get(root)).with_state(state)
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    console_error_panic_hook::set_once();
    Ok(router(env).call(req).await?)
}

#[axum_macros::debug_handler]
// #[worker::send]
pub async fn root(State(AppState { env }): State<AppState>) -> &'static str {
    let objects = env
        .bucket("BUCKET")
        .unwrap()
        .list()
        .execute()
        .await
        .unwrap();
    "Hello Axum!"
}
