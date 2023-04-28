use axum::{routing::get, Router};
use serde::{Deserialize, Serialize};

use crate::AppState;

pub mod get_world;
pub mod get_world_guilds;
pub mod get_world_kill_statistics;
pub mod get_worlds;

#[derive(Serialize, Deserialize, Debug)]
pub struct PathParams {
    world_name: String,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(get_worlds::handler))
        .route("/:world_name", get(get_world::handler))
        .route("/:world_name/guilds", get(get_world_guilds::handler))
        .route(
            "/:world_name/kill-statistics",
            get(get_world_kill_statistics::handler),
        )
        .with_state(state)
}
