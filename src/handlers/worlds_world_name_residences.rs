use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Timelike};
use itertools::Itertools;
use regex::Regex;
use reqwest::{Response, StatusCode};
use reqwest_middleware::ClientWithMiddleware;
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
    town: String,
}

impl QueryParams {
    pub fn town(&self) -> String {
        self.town.to_string()
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
#[instrument(skip(state))]
#[instrument(name = "Get Houses", skip(state))]
pub async fn get(
    State(state): State<AppState>,
    Path(path_params): Path<PathParams>,
    Query(query_params): Query<QueryParams>,
) -> Result<impl IntoResponse, ServerError> {
    let client = &state.client;
    let world_name = path_params.world_name();
    let town = query_params.town();

    let houses = get_world_residences(client, &world_name, &ResidenceType::House, &town);
    let guildhalls = get_world_residences(client, &world_name, &ResidenceType::Guildhall, &town);

    match futures::join!(houses, guildhalls) {
        (Ok(Some(mut houses)), Ok(Some(guildhalls))) => {
            houses.extend(guildhalls);
            Ok(Json(houses).into_response())
        }
        (Ok(None), _) | (_, Ok(None)) => Ok(StatusCode::NOT_FOUND.into_response()),
        (_, Err(e)) | (Err(e), _) => {
            tracing::error!("Failed to fetch residences: {:?}", e);
            Err(e.into())
        }
    }
}

#[instrument(skip(client))]
pub async fn get_world_residences(
    client: &ClientWithMiddleware,
    world_name: &str,
    residence_type: &ResidenceType,
    town: &str,
) -> Result<Option<Vec<Residence>>> {
    let response = fetch_residences_page(client, world_name, residence_type, town).await?;
    let houses = parse_residences_page(response, world_name, residence_type, town).await?;

    Ok(houses)
}

#[instrument(skip(client))]
async fn fetch_residences_page(
    client: &ClientWithMiddleware,
    world_name: &str,
    residence_type: &ResidenceType,
    town: &str,
) -> Result<Response, reqwest_middleware::Error> {
    let mut params = HashMap::new();
    params.insert("subtopic", "houses");
    params.insert("world", world_name);
    params.insert("town", town);
    let residence_string = match residence_type {
        ResidenceType::House => "houses",
        ResidenceType::Guildhall => "guildhalls",
    };
    params.insert("type", residence_string);
    let response = client.get(COMMUNITY_URL).query(&params).send().await?;

    Ok(response)
}

#[instrument(skip(response))]
async fn parse_residences_page(
    response: Response,
    world_name: &str,
    residence_type: &ResidenceType,
    town: &str,
) -> Result<Option<Vec<Residence>>> {
    let text = response.text().await?;
    let document = scraper::Html::parse_document(&text);

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
    let re = regex::Regex::new(&format!("(.*) in {town} on {world_name}")).unwrap();
    if re.find(title).is_none() {
        return Ok(None);
    }
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

    let house_id_selector = Selector::parse("input[name=\"houseid\"]").expect("Invalid selector");

    for row in house_rows {
        let house_id = row
            .select(&house_id_selector)
            .next()
            .and_then(|e| e.value().attr("value"))
            .and_then(|s| s.parse::<u32>().ok())
            .context("Failed to parse house id")?;

        // If it's an invalid town it will be `No <residence_type> found.`
        let column_count = row.text().count();
        if column_count == 1 {
            return Ok(None);
        }

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
            "auction (no bid yet)" => ResidenceStatus::AuctionNoBid,
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

                    let time = time_matches
                        .get(1)
                        .map(|t| t.as_str())
                        .and_then(|t| t.parse().ok());
                    let time_unit = time_matches.get(2).map(|u| u.as_str());

                    // current date time without minutes / seconds
                    let current_dt = chrono::Utc::now()
                        .with_minute(0)
                        .and_then(|d| d.with_second(0))
                        .and_then(|d| d.with_nanosecond(0))
                        .context("Failed to construct current time")?;

                    match (time, time_unit) {
                        (Some(time), Some(unit)) => {
                            let duration = match unit {
                                "day" | "days" => Duration::days(time),
                                "hour" | "hours" => Duration::hours(time),
                                // Because of the regex this cannot happen
                                _ => panic!("Invalid time unit"),
                            };

                            let expires_dt =
                                current_dt.checked_add_signed(duration).context(format!(
                                    "Failed to calculate expiry time `{time}` with unit `{unit}`"
                                ))?;
                            ResidenceStatus::AuctionWithBid {
                                bid: gold,
                                expiry_time: expires_dt,
                            }
                        }
                        _ => return Err(anyhow!(format!("Time and unit not found: `{}`", value))),
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

    Ok(Some(residences))
}
