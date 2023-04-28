use axum::{routing::get, Router};

use crate::AppState;

pub mod get_towns;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(get_towns::handler))
        .with_state(state)
}
