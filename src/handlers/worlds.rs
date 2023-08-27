use std::collections::HashMap;

use anyhow::{Context, Result};
use axum::{extract::State, Json};
use chrono::{prelude::*, TimeZone, Utc};
use chrono_tz::Europe::Stockholm;
use regex::Regex;
use reqwest::{Response};
use reqwest_middleware::ClientWithMiddleware;
use scraper::Selector;
use tracing::instrument;

use crate::{
    models::{GameWorldType, TransferType, World, WorldsResponse},
    prelude::*,
    AppState,
};

/// Worlds
///
#[utoipa::path(
    get,
    operation_id = "get_worlds",
    path = "/api/v1/worlds",
    responses(
        (status = 200, description = "Success", body = WorldsResponse),
        (status = 500, description = "Internal Server Error"),
        (status = 503, description = "Service Unavailable", body = PublicErrorBody)
    ),
    tag = "Worlds"
)]
#[axum::debug_handler]
#[instrument(name = "Get Worlds", skip(state))]
pub async fn get(State(state): State<AppState>) -> Result<Json<WorldsResponse>, ServerError> {
    let client = &state.client;

    let response = fetch_worlds_page(client).await.map_err(|e| {
        tracing::error!("Failed to fetch worlds page: {:?}", e);
        e
    })?;
    let worlds = parse_worlds_page(response).await.map_err(|e| {
        tracing::error!("Failed to parse worlds page: {:?}", e);
        e
    })?;

    Ok(Json(worlds))
}

#[instrument(skip(client))]
async fn fetch_worlds_page(
    client: &ClientWithMiddleware,
) -> Result<Response, reqwest_middleware::Error> {
    let mut params = HashMap::new();
    params.insert("subtopic", "worlds");
    let response = client.get(COMMUNITY_URL).query(&params).send().await?;

    Ok(response)
}

#[instrument(skip(response))]
async fn parse_worlds_page(response: Response) -> Result<WorldsResponse> {
    let text = response.text().await?;
    let document = scraper::Html::parse_document(&text);

    let selector = Selector::parse(".main-content").expect("Invalid selector for main content");
    let main_content = &document
        .select(&selector)
        .next()
        .context("ElementRef for main content not found")?;

    let tables_selector =
        Selector::parse(".TableContent").expect("Invalid selector for worlds table");
    let mut tables = main_content.select(&tables_selector);

    let mut worlds_data = WorldsResponse {
        players_online_total: 0,
        record_players: 0,
        record_date: Utc::now(),
        worlds: vec![],
    };

    let record_table = tables.next().context("Record table not found")?;
    tables.next(); // skip table
    let worlds_table = tables.next().context("Worlds table not found")?;

    // RECORD PLAYERS
    let record_html = record_table.inner_html().sanitize();
    let re = Regex::new(r"\(on (.*) CES?T\)").unwrap();
    let record_date = re
        .captures(&record_html)
        .and_then(|c| c.get(1))
        .context(format!("Record date not found in {}", record_html))?
        .as_str();

    let naive_dt = NaiveDateTime::parse_from_str(record_date, "%b %d %Y, %H:%M:%S").context(
        format!("Failed to parse online record date {}", record_date),
    )?;
    let utc_time = Stockholm
        .from_local_datetime(&naive_dt)
        .unwrap()
        .with_timezone(&Utc);
    worlds_data.record_date = utc_time;

    let re = Regex::new(r"([\d,]+)").unwrap();
    let record_players = re
        .find(&record_html)
        .context(format!("Record players not found in {}", record_html))?
        .as_str();
    let record_players: u32 = record_players
        .replace(',', "")
        .parse()
        .context(format!("Failed to parse record players {}", record_players))?;
    worlds_data.record_players = record_players;

    // WORLDS
    let world_row_relector =
        Selector::parse("tr.Odd > td, tr.Even > td").expect("Invalid selector for world row");
    let name_selector = Selector::parse("a").expect("Invalid selector for world name");
    let mut cells = worlds_table.select(&world_row_relector);
    while let (
        Some(name),
        Some(players_online),
        Some(location),
        Some(pvp_type),
        Some(battl_eye),
        Some(additional_information),
    ) = (
        cells.next(),
        cells.next(),
        cells.next(),
        cells.next(),
        cells.next(),
        cells.next(),
    ) {
        let battl_eye_selector =
            Selector::parse(".HelperDivIndicator").expect("Invalid selector for battl eye");
        let additional_information = additional_information.inner_html();

        // TODO: split tags and parse with FromStr
        let game_world_type = if additional_information.contains("experimental") {
            GameWorldType::Experimental
        } else {
            GameWorldType::Regular
        };

        let premium_required = additional_information.contains("premium");
        let transfer_type = if additional_information.contains("blocked") {
            Some(TransferType::Blocked)
        } else if additional_information.contains("locked") {
            Some(TransferType::Locked)
        } else {
            None
        };

        // TODO: Simplify ?
        let battl_eye_attr = battl_eye
            .select(&battl_eye_selector)
            .next()
            .and_then(|e| e.value().attr("onmouseover"));

        let battl_eye_date = battl_eye_attr.map_or_else(
            || -> Result<Option<NaiveDate>> { Ok(None) },
            |s| {
                if s.contains("release") {
                    return Ok(None);
                }

                let re = Regex::new(r"since (.*)\.").unwrap();

                match re.find(s) {
                    Some(mat) => {
                        let s = mat.as_str();
                        let naive_date = NaiveDate::parse_from_str(s, "since %B %d, %Y.")
                            .context(format!("Failed to parse date {}", s))?;

                        Ok(Some(naive_date))
                    }
                    None => Ok(None),
                }
            },
        )?;
        let world = World {
            name: name
                .select(&name_selector)
                .next()
                .context("World name not found")?
                .inner_html(),
            players_online_count: players_online.inner_html().replace(',', "").parse()?,
            location: location.inner_html().parse()?,
            pvp_type: pvp_type.inner_html().parse().unwrap(),
            battl_eye: !battl_eye.inner_html().is_empty(),
            battl_eye_date,
            premium_required,
            game_world_type,
            transfer_type,
        };

        worlds_data.players_online_total += world.players_online_count;
        worlds_data.worlds.push(world);
    }

    Ok(worlds_data)
}
