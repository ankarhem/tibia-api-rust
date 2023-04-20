use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::AppState;

const COMMUNITY_URL: &'static str = "https://www.tibia.com/community/";

#[derive(Serialize, Deserialize, Debug)]
pub struct PathParams {
    world: String,
}

#[axum::debug_handler]
pub async fn list_worlds(State(state): State<AppState>) -> Result<Response, StatusCode> {
    let client = state.client;

    let mut params = HashMap::new();
    params.insert("subtopic", "worlds");
    let response = client
        .get(COMMUNITY_URL)
        .query(&params)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let page_as_str = response
        .text()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let worlds =
        tibia_api::scrape_worlds(&page_as_str).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let json = Json(worlds);
    Ok(json.into_response())
}

#[axum::debug_handler]
pub async fn get_kill_statistics(
    State(state): State<AppState>,
    Path(path_params): Path<PathParams>,
) -> Result<Response, StatusCode> {
    let client = state.client;

    // Form data
    let mut params = HashMap::new();
    params.insert("subtopic", "killstatistics");
    params.insert("world", &path_params.world);

    let response = client
        .get(COMMUNITY_URL)
        .query(&params)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let page_as_str = response
        .text()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stats = tibia_api::scrape_kill_statistics(&page_as_str)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let json = Json(stats);
    Ok(json.into_response())
}
