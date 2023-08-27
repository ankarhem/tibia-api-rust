use std::collections::HashMap;

use crate::models::{GameWorldType, Location, Player, PvpType, Vocation, WorldDetails};
use crate::{prelude::*, AppState};
use anyhow::{anyhow, Context, Result};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use capitalize::Capitalize;
use chrono::{prelude::*, TimeZone, Utc};
use chrono_tz::Europe::Stockholm;
use regex::Regex;
use reqwest::{Response, StatusCode};
use reqwest_middleware::ClientWithMiddleware;
use scraper::Selector;
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Serialize, Deserialize, Debug, utoipa::IntoParams)]
pub struct WorldParams {
    /// Name of world
    #[param(example = "Antica")]
    pub world_name: String,
}

/// World
///
#[utoipa::path(
    get,
    operation_id = "get_world_details",
    path = "/api/v1/worlds/{world_name}",
    params(WorldParams),
    responses(
        (status = 200, description = "Success", body = WorldDetails),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
        (status = 503, description = "Service Unavailable", body = PublicErrorBody)
    ),
    tag = "Worlds"
)]
#[axum::debug_handler]
#[instrument(name = "Get World", skip(state))]
pub async fn get(
    State(state): State<AppState>,
    Path(path_params): Path<WorldParams>,
) -> Result<impl IntoResponse, ServerError> {
    let client = &state.client;
    let world_name = path_params.world_name.capitalize();

    let response = fetch_world_details_page(client, &world_name)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch world page: {:?}", e);
            e
        })?;
    let world_details = parse_world_details_page(response, &world_name)
        .await
        .map_err(|e| {
            tracing::error!("Failed to parse world page: {:?}", e);
            e
        })?;

    match world_details {
        Some(d) => Ok(Json(d).into_response()),
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

#[instrument(skip(client))]
pub async fn fetch_world_details_page(
    client: &ClientWithMiddleware,
    world_name: &str,
) -> Result<Response, reqwest_middleware::Error> {
    let mut params = HashMap::new();
    params.insert("subtopic", "worlds");
    params.insert("world", world_name);
    let response = client.get(COMMUNITY_URL).query(&params).send().await?;

    Ok(response)
}

#[instrument(skip(response))]
pub async fn parse_world_details_page(
    response: Response,
    world_name: &str,
) -> Result<Option<WorldDetails>> {
    let text = response.text().await?;
    let document = scraper::Html::parse_document(&text);

    let selector = Selector::parse(".main-content").expect("Invalid selector for main content");
    let main_content = &document
        .select(&selector)
        .next()
        .context("ElementRef for main content not found")?;

    let tables_selector =
        Selector::parse(".InnerTableContainer").expect("Invalid selector for worlds table");
    let mut tables = main_content.select(&tables_selector);

    // is a 404 page
    if tables.clone().count() == 1 {
        tracing::info!("World '{}' not found", world_name);
        return Ok(None);
    }

    // skip first table
    tables.next();
    let information_table = tables
        .next()
        .context(format!("Information table not found"))?;

    let cell_selector = Selector::parse("td").expect("Invalid selector for table cell");
    let mut information_cells = information_table.select(&cell_selector);

    let mut world_details = WorldDetails {
        name: world_name.to_string(),
        is_online: true,
        players_online_count: 0,
        players_online_record: 0,
        players_online_record_date: Utc::now(),
        creation_date: NaiveDate::from_ymd_opt(1, 1, 1).unwrap(),
        location: Location::Europe,
        pvp_type: PvpType::Open,
        world_quest_titles: vec![],
        battl_eye: false,
        battl_eye_date: None,
        game_world_type: GameWorldType::Regular,
        transfer_type: None,
        premium_required: false,
        players_online: vec![],
    };

    while let (Some(header), Some(value)) = (information_cells.next(), information_cells.next()) {
        match header.inner_html().as_str() {
            "Status:" => {
                let value = value.text().next().map(|s| s.trim());
                let status = match value {
                    Some("Online") => true,
                    Some("Offline") => false,
                    _ => Err(anyhow!(format!("Unexpected online status {:?}", value)))?,
                };
                world_details.is_online = status;
            }
            "Players Online:" => {
                let value = value.inner_html().replace(',', "");
                let players_online_count = value
                    .parse()
                    .context(format!("Failed to parse players online count {}", value))?;
                world_details.players_online_count = players_online_count;
            }
            "Online Record:" => {
                let record_html = value.inner_html().sanitize();
                let re = Regex::new(r"([\d,]+)").expect("Invalid regex");

                let online_record = re
                    .find(&record_html)
                    .context(format!("Online record not found"))?
                    .as_str()
                    .replace(',', "");

                let online_record: u32 = online_record
                    .parse()
                    .context(format!("Failed to parse online record {}", online_record))?;
                world_details.players_online_record = online_record;

                let re = Regex::new(r"\(on (.*) CES?T\)").unwrap();
                let record_date = re
                    .captures(&record_html)
                    .and_then(|c| c.get(1))
                    .context(format!("Record date not found in {}", record_html))?
                    .as_str();

                let naive_dt =
                    NaiveDateTime::parse_from_str(record_date, "%b %d %Y, %H:%M:%S").context(
                        format!("Failed to parse online record date {}", record_date),
                    )?;
                let utc_time = Stockholm
                    .from_local_datetime(&naive_dt)
                    .unwrap()
                    .with_timezone(&Utc);
                world_details.players_online_record_date = utc_time;
            }
            "Creation Date:" => {
                let date_html = &value.inner_html().sanitize();
                let date_html = format!("01 {date_html}");

                let naive_date = NaiveDate::parse_from_str(&date_html, "%d %B %Y")
                    .context(format!("Failed to parse creation date {}", &date_html))?;
                world_details.creation_date = naive_date;
            }
            "Location:" => {
                world_details.location = value.inner_html().parse()?;
            }
            "PvP Type:" => {
                world_details.pvp_type = value.inner_html().parse()?;
            }
            "World Quest Titles:" => {
                let mut titles = vec![];
                let title_selector = Selector::parse("a").expect("Invalid selector for titles");

                for title in value.select(&title_selector) {
                    titles.push(title.inner_html().sanitize());
                }

                world_details.world_quest_titles = titles;
            }
            "BattlEye Status:" => {
                let string = value.inner_html();
                if string.contains("release") {
                    world_details.battl_eye = true;
                } else if string.contains("since") {
                    world_details.battl_eye = true;

                    let re = Regex::new(r"since (.*)\.").expect("Invalid regex");
                    let s = re.find(&string).context("Date not found")?.as_str();
                    let naive_date = NaiveDate::parse_from_str(s, "since %B %d, %Y.")
                        .context(format!("Failed to parse BattlEye date {}", s))?;

                    world_details.battl_eye_date = Some(naive_date);
                } else {
                    world_details.battl_eye = false;
                }
            }
            "Transfer Type:" => {
                // If the header exist parsing should work
                world_details.transfer_type = Some(value.inner_html().parse()?);
            }
            "Premium Type:" => match value.inner_html().as_str() {
                "premium" => {
                    world_details.premium_required = true;
                }
                _ => {
                    world_details.premium_required = false;
                }
            },
            "Game World Type:" => {
                world_details.game_world_type =
                    value.inner_html().sanitize().to_lowercase().parse()?;
            }
            _ => {
                return Err(anyhow!(format!(
                    "Unexpected header {:?}",
                    header.inner_html()
                )))
            }
        }
    }

    // Only try to parse players table if there are players online
    if world_details.players_online_count > 0 {
        let players_online_table = tables.next().context("Players online table not found")?;
        let player_cell_selector =
            Selector::parse("tr.Odd > td, tr.Even > td").expect("Invalid selector for player cell");
        let mut player_cells = players_online_table.select(&player_cell_selector);

        while let (Some(name), Some(level), Some(vocation)) = (
            player_cells.next(),
            player_cells.next(),
            player_cells.next(),
        ) {
            let vocation_string = vocation.inner_html().sanitize();
            let vocation: Option<Vocation> = match vocation_string.as_str() {
                "None" => None,
                _ => Some(vocation_string.parse()?),
            };
            let player_name = name
                .text()
                .next()
                .context("Player name not found")?
                .to_string();
            let player = Player {
                name: player_name,
                level: level.inner_html().parse()?,
                vocation,
            };
            world_details.players_online.push(player);
        }
    }

    Ok(Some(world_details))
}
