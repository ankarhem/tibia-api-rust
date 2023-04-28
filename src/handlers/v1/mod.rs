use axum::Router;

use crate::AppState;

pub mod towns;
pub mod worlds;

pub fn router(state: AppState) -> Router {
    Router::new()
        .nest("/v1/worlds", worlds::router(state.clone()))
        .nest("/v1/towns", towns::router(state))
}
