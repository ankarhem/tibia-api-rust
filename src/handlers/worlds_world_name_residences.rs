use core::panicking::panic;
use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use capitalize::Capitalize;
use futures::{stream, StreamExt};
use reqwest::{Response, StatusCode};
use reqwest_middleware::ClientWithMiddleware;
use scraper::Selector;
use tokio::{sync::Mutex, task::JoinHandle};
use tracing::instrument;

use super::worlds_world_name::WorldParams;
use crate::{
    models::{Residence, ResidenceType},
    prelude::*,
    AppState,
};

const PARALLEL_REQUESTS: usize = 2;

/// Residences
///
#[utoipa::path(
    get,
    operation_id = "get_world_residences",
    path = "/api/v1/worlds/{world_name}/residences",
    params(WorldParams),
    responses(
        (status = 200, description = "Success", body = [Residence]),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
        (status = 503, description = "Service Unavailable", body = PublicErrorBody)
    ),
    tag = "Worlds"
)]
#[instrument(skip(state))]
#[instrument(name = "Get Houses", skip(state))]
pub async fn get(
    State(state): State<AppState>,
    Path(path_params): Path<WorldParams>,
) -> Result<impl IntoResponse, ServerError> {
    let client = &state.client;
    let world_name = path_params.world_name.capitalize();

    let residences: Arc<Mutex<Option<Vec<Residence>>>> = Arc::new(Mutex::new(Some(vec![])));

    let handle: JoinHandle<Result<Option<bool>>> = tokio::spawn(async move {
        let residence_type = ResidenceType::House;
        let client = client.clone();
        unsafe {}
        let response = fetch_residences_page(&client, &world_name, &residence_type)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch residences page: {:?}", e);
                e
            })?;
        let mut houses = parse_residences_page(response, &residence_type)
            .await
            .map_err(|e| {
                tracing::error!("Failed to parse residences page: {:?}", e);
                e
            })?;

        let mut residences = residences.clone().lock().await;
        match (*residences, houses) {
            (Some(mut residences), Some(mut houses)) => {
                residences.append(&mut houses);
                Ok(Some(true))
            }
            (_, None) => {
                *residences = None;
                Ok(None)
            }
            _ => {
                *residences = None;
                Ok(None)
            }
        }
    });

    let mut residences = residences.lock().await;
    Ok(Json(*residences))
}

#[instrument(skip(client))]
async fn fetch_residences_page(
    client: &ClientWithMiddleware,
    world_name: &str,
    residence_type: &ResidenceType,
) -> Result<Response, reqwest_middleware::Error> {
    let mut params = HashMap::new();
    params.insert("subtopic", "houses");
    params.insert("world", world_name);
    let residence_string = match residence_type {
        ResidenceType::House => "houses",
        ResidenceType::Guildhall => "guildhalls",
    };
    params.insert("type", &residence_string);
    let response = client.get(COMMUNITY_URL).query(&params).send().await?;

    Ok(response)
}

#[instrument(skip(response))]
async fn parse_residences_page(
    response: Response,
    residence_type: &ResidenceType,
) -> Result<Option<Vec<Residence>>> {
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
    if tables.clone().count() != 3 {
        return Ok(None);
    }

    let row_selector = Selector::parse("tr").expect("Selector to be valid");
    let house_rows = tables.next().unwrap().select(&row_selector).skip(1);

    let mut residences = vec![];

    for row in house_rows {
        let (name, size, rent, status) = row
            .text()
            .collect_tuple()
            .context("Residence row does not contain 4 columns")?;

        let number_re = regex::Regex::new(r"(\d+)").unwrap();
        let size = number_re
            .captures(size)
            .and_then(|s| s.get(1))
            .and_then(|s| s.as_str().parse().ok())
            .context(format!("Failed to parse size: {}", size))?;

        let rent = number_re
            .captures(rent)
            .and_then(|s| s.get(1))
            .and_then(|s| s.as_str().parse::<u32>().ok())
            .map(|s| s * 1000)
            .context(format!("Failed to parse rent: {}", rent))?;

        let residence = Residence {
            residence_type: *residence_type,
            name: name.to_string().sanitize(),
            size,
            rent,
            status: status.to_string().sanitize(),
        };

        residences.push(residence)
    }

    Ok(Some(residences))
}
