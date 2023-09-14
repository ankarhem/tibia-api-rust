use std::collections::HashMap;

use anyhow::{Context, Result};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use reqwest::{Response, StatusCode};
use reqwest_middleware::ClientWithMiddleware;
use scraper::Selector;
use tracing::instrument;

use super::worlds_world_name::PathParams;
use crate::{
    models::{KillStatistics, KilledAmounts, RaceKillStatistics},
    prelude::*,
    AppState,
};

/// Kill Statistics
///
#[utoipa::path(
    get,
    operation_id = "get_world_kill_statistics",
    path = "/api/v1/worlds/{world_name}/kill-statistics",
    params(PathParams),
    responses(
        (status = 200, description = "Success", body = KillStatistics),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
        (status = 503, description = "Service Unavailable", body = PublicErrorBody)
    ),
    tag = "Worlds"
)]
#[instrument(skip(state))]
#[instrument(name = "Get Kill Statistics", skip(state))]
pub async fn get(
    State(state): State<AppState>,
    Path(path_params): Path<PathParams>,
) -> Result<impl IntoResponse, ServerError> {
    let client = &state.client;
    let world_name = path_params.world_name();

    let response = fetch_killstatistics_page(client, &world_name)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch kill statistics page: {:?}", e);
            e
        })?;
    let guilds = parse_killstatistics_page(response).await.map_err(|e| {
        tracing::error!("Failed to parse kill statistics page: {:?}", e);
        e
    })?;

    match guilds {
        Some(g) => Ok(Json(g).into_response()),
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

#[instrument(skip(client))]
async fn fetch_killstatistics_page(
    client: &ClientWithMiddleware,
    world_name: &str,
) -> Result<Response, reqwest_middleware::Error> {
    let mut params = HashMap::new();
    params.insert("subtopic", "killstatistics");
    params.insert("world", world_name);
    let response = client.get(COMMUNITY_URL).query(&params).send().await?;

    Ok(response)
}

#[instrument(skip(response))]
async fn parse_killstatistics_page(response: Response) -> Result<Option<KillStatistics>> {
    let text = response.text().await?;
    let document = scraper::Html::parse_document(&text);

    let selector = Selector::parse(".main-content").expect("Selector to be valid");
    let main_content = document
        .select(&selector)
        .next()
        .context("ElementRef for main content not found")?;

    let table_cell_selector = Selector::parse("#KillStatisticsTable tr.DataRow > td")
        .expect("Invalid selector for kill statistics table");

    let mut cells = main_content.select(&table_cell_selector);

    // assume 404
    if cells.clone().count() == 0 {
        return Ok(None);
    }

    let mut stats: KillStatistics = KillStatistics {
        races: vec![],
        total_last_day: KilledAmounts {
            killed_players: 0,
            killed_by_players: 0,
        },
        total_last_week: KilledAmounts {
            killed_players: 0,
            killed_by_players: 0,
        },
    };

    while let (Some(name), Some(kp_day), Some(kbp_day), Some(kp_week), Some(kbp_week)) = (
        cells.next().map(|c| c.inner_html()),
        cells.next().map(|c| c.inner_html()),
        cells.next().map(|c| c.inner_html()),
        cells.next().map(|c| c.inner_html()),
        cells.next().map(|c| c.inner_html()),
    ) {
        // handle the last row
        if name == "Total" {
            stats.total_last_day = KilledAmounts {
                killed_players: kp_day.parse()?,
                killed_by_players: kbp_day.parse()?,
            };

            stats.total_last_week = KilledAmounts {
                killed_players: kp_week.parse()?,
                killed_by_players: kbp_week.parse()?,
            };
            continue;
        }

        let last_day = KilledAmounts {
            killed_players: kp_day.parse()?,
            killed_by_players: kbp_day.parse()?,
        };

        let last_week = KilledAmounts {
            killed_players: kp_week.parse()?,
            killed_by_players: kbp_week.parse()?,
        };

        stats.races.push(RaceKillStatistics {
            race: name.to_string(),
            last_day,
            last_week,
        })
    }

    Ok(Some(stats))
}
