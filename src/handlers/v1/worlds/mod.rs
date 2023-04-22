use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{handlers::ApiError, AppState};

const COMMUNITY_URL: &'static str = "https://www.tibia.com/community/";

#[derive(Serialize, Deserialize, Debug)]
pub struct PathParams {
    world_name: String,
}

/// List all worlds.
///
#[utoipa::path(
    get,
    path = "/api/v1/worlds",
    responses(
        (status = 200, description = "Success", body = WorldsData),
        (status = 500, description = "Internal Server Error", body = ApiError),
    ),
    tag = "Worlds"
)]
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
            Some(tibia_api::ParseError::UnexpectedPageContent(s)) => {
                Err(ApiError::internal_server_error(s))
            }
            _ => Err(ApiError::internal_server_error(
                "Failed to scrape source data",
            )),
        },
    }
}

/// Show details for a world.
///
#[utoipa::path(
    get,
    path = "/api/v1/worlds/{world_name}",
    params(
        ("world_name" = String, Path, description = "World name", example = "Antica")
    ),
    responses(
        (status = 200, description = "Success", body = WorldDetails),
        (status = 404, description = "Not Found", body = ApiError),
        (status = 500, description = "Internal Server Error", body = ApiError),
    ),
    tag = "Worlds"
)]
#[axum::debug_handler]
pub async fn get_world_details(
    State(state): State<AppState>,
    Path(path_params): Path<PathParams>,
) -> Result<Response, ApiError> {
    let client = state.client;

    let mut params = HashMap::new();
    params.insert("subtopic", "worlds");
    params.insert("world", &path_params.world_name);
    let response = client
        .get(COMMUNITY_URL)
        .query(&params)
        .send()
        .await
        .map_err(|_| ApiError::internal_server_error("Could not connect to tibia.com"))?;

    let page_as_str = response.text().await.map_err(|_| {
        ApiError::internal_server_error("Could not decode source response body from tibia.com")
    })?;

    let world_details = tibia_api::scrape_world_details(&page_as_str);

    match world_details {
        Ok(world_details) => {
            let json = Json(world_details);
            Ok(json.into_response())
        }
        Err(e) => match e.downcast_ref() {
            Some(tibia_api::ParseError::Is404) => Err(ApiError::not_found("World not found")),
            Some(tibia_api::ParseError::UnexpectedPageContent(s)) => {
                Err(ApiError::internal_server_error(s))
            }
            _ => Err(ApiError::internal_server_error(
                "Failed to scrape source data",
            )),
        },
    }
}

/// List killstatistics for a world.
///
#[utoipa::path(
    get,
    path = "/api/v1/worlds/{world_name}/kill-statistics",
    params(
        ("world_name" = String, Path, description = "World name", example = "Antica")
    ),
    responses(
        (status = 200, description = "Success", body = [MonsterStats]),
        (status = 404, description = "Not Found", body = ApiError),
        (status = 500, description = "Internal Server Error", body = ApiError),
    ),
    tag = "Worlds"
)]
#[axum::debug_handler]
pub async fn get_world_kill_statistics(
    State(state): State<AppState>,
    Path(path_params): Path<PathParams>,
) -> Result<Response, ApiError> {
    let client = state.client;

    // Form data
    let mut params = HashMap::new();
    params.insert("subtopic", "killstatistics");
    params.insert("world", &path_params.world_name);

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
            Some(tibia_api::ParseError::UnexpectedPageContent(s)) => {
                Err(ApiError::internal_server_error(s))
            }
            _ => Err(ApiError::internal_server_error(
                "Unknown error while parsing source data",
            )),
        },
    }
}

/// List all guilds for a world.
///
#[utoipa::path(
    get,
    path = "/api/v1/worlds/{world_name}/guilds",
    params(
        ("world_name" = String, Path, description = "World name", example = "Antica")
    ),
    responses(
        (status = 200, description = "Success", body = [Guild]),
        (status = 404, description = "Not Found", body = ApiError),
        (status = 500, description = "Internal Server Error", body = ApiError),
    ),
    tag = "Worlds"
)]
#[axum::debug_handler]
pub async fn get_world_guilds(
    State(state): State<AppState>,
    Path(path_params): Path<PathParams>,
) -> Result<Response, ApiError> {
    let client = state.client;

    // Form data
    let mut params = HashMap::new();
    params.insert("subtopic", "guilds");
    params.insert("world", &path_params.world_name);

    let response = client
        .get(COMMUNITY_URL)
        .query(&params)
        .send()
        .await
        .map_err(|_| ApiError::internal_server_error("Could not connect to tibia.com"))?;

    let page_as_str = response.text().await.map_err(|_| {
        ApiError::internal_server_error("Could not decode source response body from tibia.com")
    })?;

    println!("{page_as_str}");
    let guilds = tibia_api::scrape_guilds(&page_as_str);

    match guilds {
        Ok(guilds) => {
            let json = Json(guilds);
            Ok(json.into_response())
        }
        Err(e) => match e.downcast_ref() {
            Some(tibia_api::ParseError::Is404) => Err(ApiError::not_found("World not found")),
            Some(tibia_api::ParseError::UnexpectedPageContent(s)) => {
                Err(ApiError::internal_server_error(s))
            }
            _ => Err(ApiError::internal_server_error(
                "Unknown error while parsing source data",
            )),
        },
    }
}
