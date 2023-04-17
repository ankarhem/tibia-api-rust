use axum::{routing::get, Router};
use reqwest;

mod handlers;
pub use crate::handlers::worlds;

#[derive(Clone)]
pub struct AppState {
    client: reqwest::Client,
}

impl AppState {
    fn new() -> AppState {
        let client = reqwest::Client::new();
        AppState { client }
    }
}

#[tokio::main]
async fn main() {
    let state = AppState::new();

    let app = Router::new().route("/", get(root)).route(
        "/worlds/:world/kill_statistics",
        get(worlds::get_kill_statistics),
    );

    let server = axum::Server::bind(&"0.0.0.0:7032".parse().unwrap())
        .serve(app.with_state(state).into_make_service());
    let addr = server.local_addr();

    println!("Listening on {addr}");

    server.await.unwrap();
}

async fn root() -> &'static str {
    "Hello world"
}
