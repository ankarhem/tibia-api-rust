use std::collections::HashMap;

use crate::prelude::*;
use anyhow::{Context, Result};
use axum::{extract::State, Json};
use reqwest::Client;
use scraper::Selector;
use tracing::instrument;

use crate::AppState;

#[axum::debug_handler]
#[instrument(skip(state))]
pub async fn get_towns(State(state): State<AppState>) -> Result<Json<Vec<String>>, PublicError> {
    let client = &state.client;

    let page = fetch_towns_page(client).await.map_err(|e| {
        tracing::error!("Failed to fetch towns page: {:?}", e);
        e
    })?;
    let towns = parse_towns_page(page).await.map_err(|e| {
        tracing::error!("Failed to parse towns page: {:?}", e);
        e
    })?;

    println!("Towns: {:?}", towns);

    Ok(Json(towns))
}

#[instrument(skip(client))]
pub async fn fetch_towns_page(client: &Client) -> Result<reqwest::Response, reqwest::Error> {
    let mut params = HashMap::new();
    params.insert("subtopic", "houses");

    let response = client.get(COMMUNITY_URL).query(&params).send().await?;
    Ok(response)
}

#[instrument(skip(response))]
pub async fn parse_towns_page(response: reqwest::Response) -> Result<Vec<String>> {
    let text = response.text().await?;
    let document = scraper::Html::parse_document(&text);

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

    let towns_row_selector =
        Selector::parse("tr > td[valign=\"top\"").expect("Invalid selector for towns row");
    let towns_row = table
        .select(&towns_row_selector)
        .next()
        .context("Towns row not found")?;

    let mut towns: Vec<String> = vec![];
    let town_selector = Selector::parse("label").expect("Invalid selector for town");
    for town in towns_row.select(&town_selector) {
        let town_name = town.text().collect::<String>();

        if !town_name.is_empty() {
            towns.push(town_name.sanitize());
        }
    }

    Ok(towns)
}
