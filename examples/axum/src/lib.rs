use std::sync::Arc;

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_service::Service;
use worker::{event, kv::KvStore, Context, Env, HttpRequest, Request, Response};

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> worker::Result<Response> {
    console_error_panic_hook::set_once();

    let http_request = HttpRequest::from(req);

    let response = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user))
        // .with_state(AppState {
        //     kv: Arc::new(env.kv("KV").unwrap()),
        // })
        .call(http_request)
        .await
        .unwrap();

    Ok(response.into())
}

pub async fn root() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

pub async fn create_user(Json(payload): Json<CreateUser>) -> impl IntoResponse {
    let user = User {
        id: 1,
        first_name: payload.first_name,
        last_name: payload.last_name,
    };

    (StatusCode::CREATED, Json(user))
}

#[derive(Clone)]
pub struct AppState {
    kv: Arc<KvStore>,
}

#[derive(Deserialize)]
pub struct CreateUser {
    first_name: String,
    last_name: String,
}

#[derive(Serialize)]
struct User {
    id: u32,
    first_name: String,
    last_name: String,
}
