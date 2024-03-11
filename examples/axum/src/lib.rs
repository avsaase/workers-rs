use axum::{http::StatusCode, routing::get, Router};
use tower_service::Service;
use worker::{event, Body, Context, Env, Request, Response};

fn router() -> Router {
    Router::new().route("/", get(root))
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> worker::Result<Response> {
    console_error_panic_hook::set_once();

    let response = router().route("/", get(root)).call(req).await.unwrap();

    Ok(response.map(Body::new))
}

pub async fn root() -> StatusCode {
    StatusCode::OK
}
