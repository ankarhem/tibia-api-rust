use anyhow::{Context, Result};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{Duration, Timelike};
use itertools::Itertools;
use regex::Regex;
use reqwest::Response;

use futures::stream::StreamExt;
use scraper::Selector;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::worlds_world_name::PathParams;
use crate::{
    models::{Residence, ResidenceStatus, ResidenceType},
    prelude::*,
    AppState,
};

#[derive(Serialize, Deserialize, Debug, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct QueryParams {
    /// The town for which to fetch residences
    #[param(example = "Thais")]
    town: Option<String>,
    /// Filter residences by type
    #[serde(rename = "type")]
    residence_type: Option<ResidenceType>,
}

impl QueryParams {
    pub fn town(&self) -> Option<String> {
        self.town.clone()
    }

    pub fn residence_type(&self) -> Option<ResidenceType> {
        self.residence_type
    }
}

/// Residences
///
#[utoipa::path(
    get,
    operation_id = "get_world_residences",
    path = "/api/v1/worlds/{world_name}/residences",
    params(PathParams, QueryParams),
    responses(
        (status = 200, description = "Success", body = [Residence]),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
        (status = 503, description = "Service Unavailable", body = PublicErrorBody)
    ),
    tag = "Worlds"
)]
#[instrument(name = "Get Residences", skip(state))]
pub async fn get<S: Client>(
    State(state): State<AppState<S>>,
    Path(path_params): Path<PathParams>,
    Query(query_params): Query<QueryParams>,
) -> Result<Json<Vec<Residence>>, ServerError> {
    let client = &state.client;
    let world_name = path_params.world_name();
    let towns = match query_params.town() {
        Some(t) => vec![t],
        None => {
            let towns = state.towns.lock().unwrap();
            towns.clone()
        }
    };
    let residence_types = query_params
        .residence_type()
        .map(|t| vec![t])
        .unwrap_or(vec![ResidenceType::House, ResidenceType::Guildhall]);

    let mut combinations = Vec::with_capacity(towns.len() * residence_types.len());
    for town in &towns {
        for residence_type in &residence_types {
            combinations.push((*residence_type, town.to_string()))
        }
    }

    // create an iterator of futures to execute
    let futures =
        (0..combinations.len()).map(|n| {
            let combination = combinations.get(n).unwrap().clone();
            let world_name = world_name.clone();
            async move {
                get_world_residences(client, &world_name, &combination.0, &combination.1).await
            }
        });

    // create a buffered stream that will execute up to 10 futures in parallel
    // (without preserving the order of the results)
    let stream = futures::stream::iter(futures).buffer_unordered(10);

    // wait for all futures to complete
    let results = stream.collect::<Vec<_>>().await;

    let residences = results
        .into_iter()
        .flatten_ok()
        .collect::<Result<Vec<Residence>, ServerError>>()
        .map_err(|e| {
            tracing::error!("Could not get residences: {:?}", e);
            e
        })?;

    Ok(Json(residences))
}

#[instrument(skip(client))]
pub async fn get_world_residences<S: Client>(
    client: &S,
    world_name: &str,
    residence_type: &ResidenceType,
    town: &str,
) -> Result<Vec<Residence>, ServerError> {
    let response = client
        .fetch_residences_page(world_name, residence_type, town)
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to residences for {world_name}, {:?}, {town}: {:?}",
                residence_type,
                e
            );
            e
        })?;
    let houses = parse_residences_page(response, world_name, residence_type, town)
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to parse residence page for {world_name}, {:?}, {town}: {:?}",
                residence_type,
                e
            );
            e
        })?;

    Ok(houses)
}

#[instrument(skip(response))]
async fn parse_residences_page(
    response: Response,
    world_name: &str,
    residence_type: &ResidenceType,
    town: &str,
) -> Result<Vec<Residence>, ServerError> {
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

    let header_selector = Selector::parse(".Text").expect("Selector to be invalid");
    let title = main_content
        .select(&header_selector)
        .next()
        .context("ElementRef for title not found")?;
    let title = title.text().next().context("Could not get title text")?;

    // If this doesn't match, a complex (invalid) town has been passed
    // and we should 404
    let re =
        regex::Regex::new(&format!("(.*) in {town} on {world_name}")).context("Invalid regex")?;
    if re.find(title).is_none() {
        return Err(TibiaError::NotFound)?;
    }
    let table_selector =
        Selector::parse(".TableContainer table.TableContent").expect("Selector to be valid");
    let mut tables = main_content.select(&table_selector);

    // assume 404
    if tables.clone().count() != 3 {
        return Err(TibiaError::NotFound)?;
    }

    let row_selector = Selector::parse("tr").expect("Selector to be valid");
    let house_rows = tables.next().unwrap().select(&row_selector).skip(1);

    let towns_selector =
        Selector::parse("input[name=town]").expect("Invalid selector for towns row");
    let towns = tables
        .last()
        .unwrap()
        .select(&towns_selector)
        .map(|e| e.value().attr("value"))
        .collect::<Option<Vec<_>>>()
        .context("Failed to parse towns")?;

    let towns: Vec<String> = towns.iter().map(|t| t.to_string().sanitize()).collect();

    if !towns.contains(&town.to_string()) {
        return Err(TibiaError::NotFound)?;
    }

    let mut residences = vec![];

    let house_id_selector = Selector::parse("input[name=\"houseid\"]").expect("Invalid selector");

    let column_count = house_rows.clone().next().map(|r| r.text().count());
    if let Some(1) = column_count {
        return Ok(vec![]);
    }

    for row in house_rows {
        let house_id = row
            .select(&house_id_selector)
            .next()
            .context("House id input not found")?;
        let house_id = house_id
            .value()
            .attr("value")
            .and_then(|s| s.parse::<u32>().ok())
            .context("Could not parse house id {house_id}")?;

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

        let value = status.to_string().sanitize();
        let status = match value.as_str() {
            "rented" => ResidenceStatus::Rented,
            "auctioned (no bid yet)" => ResidenceStatus::AuctionNoBid,
            _ => {
                let gold_re = Regex::new(r"(\d+) gold").expect("Invalid residence gold regex");
                let gold_str = gold_re
                    .captures(&value)
                    .and_then(|m| m.get(1))
                    .map(|g| g.as_str())
                    .context(format!("Expected gold in residence status: `{}`", value))?;
                let gold = gold_str
                    .parse::<u32>()
                    .context(format!("Failed to parse gold `{:?}`", gold_str))?;

                if value.contains("finished") {
                    ResidenceStatus::AuctionFinished { bid: gold }
                } else {
                    let time_re = Regex::new(r"(\d+) (days?|hours?) left")
                        .expect("Invalid residence time regex");
                    let time_matches = time_re
                        .captures(&value)
                        .context(format!("Time not found: `{}`", value))?;

                    let time: i64 = time_matches
                        .get(1)
                        .map(|t| t.as_str())
                        .and_then(|t| t.parse().ok())
                        .context("Could not parse time")?;
                    let time_unit = time_matches
                        .get(2)
                        .map(|u| u.as_str())
                        .context("Could not parse time unit")?;

                    let current_dt = chrono::Utc::now()
                        .with_minute(0)
                        .and_then(|d| d.with_second(0))
                        .and_then(|d| d.with_nanosecond(0))
                        .context("Failed to construct current time")?;

                    let current_hour = current_dt.hour();
                    // if unit is days, set hour to 8 (utc server save)
                    // otherwise we need to add an hour (0h30min left => set min 0 and add hour)
                    let current_dt = match time_unit {
                        "day" | "days" => {
                            current_dt.with_hour(8).context("Failed to set hour to 8")?
                        }
                        _ => current_dt
                            .with_hour(current_hour + 1)
                            .context("Failed to add hour")?,
                    };

                    let duration = match time_unit {
                        "day" | "days" => Duration::days(time),
                        "hour" | "hours" => Duration::hours(time),
                        // Because of the regex this cannot happen
                        _ => panic!("Invalid time unit"),
                    };

                    let expires_dt = current_dt.checked_add_signed(duration).context(format!(
                        "Failed to calculate expiry time `{time}` with unit `{time_unit}`"
                    ))?;
                    ResidenceStatus::AuctionWithBid {
                        bid: gold,
                        expiry_time: expires_dt,
                    }
                }
            }
        };

        let residence = Residence {
            id: house_id,
            residence_type: *residence_type,
            name: name.to_string().sanitize(),
            size,
            rent,
            status,
            town: town.to_string(),
        };

        residences.push(residence)
    }

    Ok(residences)
}
