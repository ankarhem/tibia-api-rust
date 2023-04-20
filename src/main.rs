use std::collections::HashMap;

use axum::{
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};

mod handlers;
pub use crate::handlers::v1;

#[derive(Clone)]
pub struct AppState {
    client: reqwest::Client,
}

impl AppState {
    fn new() -> AppState {
        // user agent + encoding is required otherwise they block the request
        // encoding is added automatically due to optional features gzip, deflate and brotli enabled
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/113.0")
            .brotli(true)
            .deflate(true)
            .gzip(true)
            .build()
            .expect("Client::builder()");
        AppState { client }
    }
}

#[tokio::main]
async fn main() {
    let state = AppState::new();

    let app = Router::new()
        .route("/__healthcheck", get(healthcheck))
        .route(
            "/api/v1/worlds/:world_name/kill-statistics",
            get(v1::worlds::get_kill_statistics),
        )
        .route("/api/v1/worlds", get(v1::worlds::list_worlds));

    let server = axum::Server::bind(&"0.0.0.0:7032".parse().unwrap())
        .serve(app.with_state(state).into_make_service());
    let addr = server.local_addr();

    println!("Listening on {addr}");

    server.await.unwrap();
}

async fn healthcheck() -> Response {
    let mut resp = HashMap::new();
    resp.insert("ok", true);
    Json(resp).into_response()
}
