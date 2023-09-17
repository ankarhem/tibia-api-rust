use anyhow::{Context, Result};
use axum::{
    extract::{Path, State},
    Json,
};
use reqwest::{Response};
use scraper::Selector;
use tracing::instrument;

use super::worlds_world_name::PathParams;
use crate::{models::Guild, prelude::*, AppState};

/// Guilds
///
#[utoipa::path(
    get,
    operation_id = "get_world_guilds",
    path = "/api/v1/worlds/{world_name}/guilds",
    params(PathParams),
    responses(
        (status = 200, description = "Success", body = [Guild]),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
        (status = 503, description = "Service Unavailable", body = PublicErrorBody)
    ),
    tag = "Worlds"
)]
#[instrument(name = "Get Guilds", skip(state))]
pub async fn get<S: Client>(
    State(state): State<AppState<S>>,
    Path(path_params): Path<PathParams>,
) -> Result<Json<Vec<Guild>>, ServerError> {
    let client = &state.client;
    let world_name = path_params.world_name();

    let response = client.fetch_guilds_page(&world_name).await.map_err(|e| {
        tracing::error!("Failed to fetch guilds page: {:?}", e);
        e
    })?;
    let guilds = parse_guilds_page(response).await.map_err(|e| {
        tracing::error!("Failed to parse guilds page: {:?}", e);
        e
    })?;

    Ok(Json(guilds))
}

#[instrument(skip(response))]
async fn parse_guilds_page(response: Response) -> Result<Vec<Guild>, ServerError> {
    let text = response.text().await?;
    let document = scraper::Html::parse_document(&text);

    let title_selector = Selector::parse("title").expect("Invalid selector for title");
    let title = document
        .select(&title_selector)
        .next()
        .and_then(|t| t.text().next())
        .unwrap_or_default();

    if MAINTENANCE_TITLE == title {
        return Err(TibiaError::Maintenance)?;
    };

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
        return Err(TibiaError::NotFound)?;
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

    Ok(guilds)
}
