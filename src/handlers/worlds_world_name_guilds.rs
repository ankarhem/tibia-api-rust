use std::collections::HashMap;

use anyhow::{Context, Result};
use axum::extract::State;
use axum::Json;
use axum::{extract::Path, response::IntoResponse};
use capitalize::Capitalize;
use reqwest::{Client, Response, StatusCode};
use scraper::Selector;
use tracing::instrument;

use super::worlds_world_name::WorldParams;
use crate::models::Guild;
use crate::{prelude::*, AppState};

/// Guilds
///
#[utoipa::path(
    get,
    operation_id = "get_world_guilds",
    path = "/api/v1/worlds/{world_name}/guilds",
    params(WorldParams),
    responses(
        (status = 200, description = "Success", body = [Guild]),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
        (status = 503, description = "Service Unavailable", body = PublicErrorBody)
    ),
    tag = "Worlds"
)]
#[instrument(skip(state))]
#[instrument(name = "Get Guilds", skip(state))]
pub async fn get(
    State(state): State<AppState>,
    Path(path_params): Path<WorldParams>,
) -> Result<impl IntoResponse, ServerError> {
    let client = &state.client;
    let world_name = path_params.world_name.capitalize();

    let response = fetch_guilds_page(client, &world_name).await.map_err(|e| {
        tracing::error!("Failed to fetch guilds page: {:?}", e);
        e
    })?;
    let guilds = parse_guilds_page(response).await.map_err(|e| {
        tracing::error!("Failed to parse guilds page: {:?}", e);
        e
    })?;

    match guilds {
        Some(g) => Ok(Json(g).into_response()),
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

#[instrument(skip(client))]
async fn fetch_guilds_page(client: &Client, world_name: &str) -> Result<Response> {
    let mut params = HashMap::new();
    params.insert("subtopic", "guilds");
    params.insert("world", world_name);
    let response = client.get(COMMUNITY_URL).query(&params).send().await?;

    Ok(response)
}

#[instrument(skip(response))]
async fn parse_guilds_page(response: Response) -> Result<Option<Vec<Guild>>> {
    let text = response.text().await?;
    let document = scraper::Html::parse_document(&text);

    let selector = Selector::parse(".main-content").expect("Selector to be valid");
    let main_content = document
        .select(&selector)
        .next()
        .context("ElementRef for main content not found")?;

    let table_selector =
        Selector::parse(".TableContainer table.TableContent").expect("Selector to be valid");
    let mut tables = main_content.select(&table_selector);

    // assume 404
    if tables.clone().count() != 2 {
        return Ok(None);
    }

    let mut guilds = vec![];

    let row_selector = Selector::parse("tr:not(:first-child)").expect("Invalid selector for rows");
    let cell_selector = Selector::parse("td").expect("Invalid selector for cells");
    let img_selector = Selector::parse("img").expect("Invalid selector for guild logo");

    for i in 0..2 {
        let table = tables.next().context("Guilds table not found")?;

        let rows = table.select(&row_selector);
        for row in rows {
            let mut cells = row.select(&cell_selector);
            let logo = cells
                .next()
                .context("Logo cell not found")?
                .select(&img_selector)
                .next()
                .and_then(|img| img.value().attr("src").map(|src| src.to_string()));

            let mut name_description_iterator = cells
                .next()
                .context("Name/description cell not found")?
                .text()
                .take(2);

            let name = name_description_iterator
                .next()
                .context("Guild name not found")?
                .to_string();

            let description = name_description_iterator.next().map(|s| s.to_string());

            guilds.push(Guild {
                logo,
                name,
                description,
                active: i == 0,
            });
        }
    }

    Ok(Some(guilds))
}