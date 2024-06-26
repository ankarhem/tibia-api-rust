use crate::prelude::*;
use anyhow::{Context, Result};
use axum::{extract::State, Json};
use scraper::Selector;
use tracing::instrument;

use crate::AppState;

/// Towns
///
#[utoipa::path(
    get,
    operation_id = "get_towns",
    path = "/api/v1/towns",
    responses(
        (status = 200, description = "Success", body = [String], example = json!([
            "Ab\'Dendriel",
            "Ankrahmun",
            "Carlin",
            "Darashia",
            "Edron",
            "Farmine",
            "Gray Beach",
            "Issavi",
            "Kazordoon",
            "Liberty Bay",
            "Moonfall",
            "Port Hope",
            "Rathleton",
            "Silvertides",
            "Svargrond",
            "Thais",
            "Venore",
            "Yalahar",
        ])),
        (status = 500, description = "Internal Server Error"),
        (status = 503, description = "Service Unavailable", body = PublicErrorBody)
    ),
    tag = "Towns"
)]
#[instrument(name = "Get Towns", skip(state))]
pub async fn get<S: Client>(
    State(state): State<AppState<S>>,
) -> Result<Json<Vec<String>>, ServerError> {
    let client = &state.client;

    let page = client.fetch_towns_page().await.map_err(|e| {
        tracing::error!("Failed to fetch towns page: {:?}", e);
        e
    })?;

    let towns = parse_towns_page(page).await.map_err(|e| {
        tracing::error!("Failed to parse towns page: {:?}", e);
        e
    })?;

    match state.towns.lock() {
        Ok(mut guard) => {
            guard.clone_from(&towns);
        }
        Err(_poisoned) => Err(anyhow::anyhow!("Mutex poisoned"))?,
    }

    Ok(Json(towns))
}

#[instrument(skip(page))]
async fn parse_towns_page(page: reqwest::Response) -> Result<Vec<String>, ServerError> {
    let text = page.text().await?;
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

    let selector = Selector::parse(".main-content").expect("Invalid selector for main content");
    let main_content = &document
        .select(&selector)
        .next()
        .context("ElementRef for main content not found")?;

    let tables_selector =
        Selector::parse("#houses table.TableContent").expect("Invalid selector for towns table");
    let table = main_content
        .select(&tables_selector)
        .last()
        .context("Towns table not found")?;

    let towns_selector =
        Selector::parse("input[name=town]").expect("Invalid selector for towns row");
    let towns = table
        .select(&towns_selector)
        .map(|e| e.value().attr("value"))
        .collect::<Option<Vec<_>>>()
        .context("Failed to parse towns")?;

    let towns = towns.iter().map(|t| t.to_string().sanitize()).collect();

    Ok(towns)
}
