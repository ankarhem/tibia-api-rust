use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

use super::ApiError;

const COMMUNITY_URL: &'static str = "https://www.tibia.com/community/";

#[derive(Serialize, Deserialize, Debug)]
pub struct PathParams {
    world: String,
}

#[axum::debug_handler]
pub async fn list_worlds(State(state): State<AppState>) -> Result<Response, ApiError> {
    let client = state.client;

    let mut params = HashMap::new();
    params.insert("subtopic", "worlds");
    let response = client
        .get(COMMUNITY_URL)
        .query(&params)
        .send()
        .await
        .map_err(|_| ApiError::internal_server_error("Could not connect to tibia.com"))?;

    let page_as_str = response.text().await.map_err(|_| {
        ApiError::internal_server_error("Could not decode source response body from tibia.com")
    })?;

    let worlds = tibia_api::scrape_worlds(&page_as_str);

    match worlds {
        Ok(worlds) => {
            let json = Json(worlds);
            Ok(json.into_response())
        }
        Err(e) => match e.downcast_ref() {
            Some(tibia_api::ParseError::Is404) => Err(ApiError::not_found("World not found")),
            Some(tibia_api::ParseError::NoneValueReceived) => Err(ApiError::internal_server_error(
                "Unable to parse unexpected response from tibia.com",
            )),
            _ => Err(ApiError::internal_server_error(
                "Failed to scrape source data",
            )),
        },
    }
}

#[axum::debug_handler]
pub async fn get_kill_statistics(
    State(state): State<AppState>,
    Path(path_params): Path<PathParams>,
) -> Result<Response, ApiError> {
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
        .map_err(|_| ApiError::internal_server_error("Could not connect to tibia.com"))?;

    let page_as_str = response.text().await.map_err(|_| {
        ApiError::internal_server_error("Could not decode source response body from tibia.com")
    })?;

    let stats = tibia_api::scrape_kill_statistics(&page_as_str);

    match stats {
        Ok(stats) => {
            let json = Json(stats);
            Ok(json.into_response())
        }
        Err(e) => match e.downcast_ref() {
            Some(tibia_api::ParseError::Is404) => Err(ApiError::not_found("World not found")),
            Some(tibia_api::ParseError::NoneValueReceived) => Err(ApiError::internal_server_error(
                "Unable to parse unexpected response from tibia.com",
            )),
            _ => Err(ApiError::internal_server_error(
                "Failed to scrape source data",
            )),
        },
    }
}
