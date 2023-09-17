use anyhow::{Context, Result};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use reqwest::{Response, StatusCode};
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
#[instrument(name = "Get Kill Statistics", skip(state))]
pub async fn get<S: Client>(
    State(state): State<AppState<S>>,
    Path(path_params): Path<PathParams>,
) -> Result<impl IntoResponse, ServerError> {
    let client = &state.client;
    let world_name = path_params.world_name();

    let response = client
        .fetch_killstatistics_page(&world_name)
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

#[instrument(skip(response))]
async fn parse_killstatistics_page(
    response: Response,
) -> Result<Option<KillStatistics>, ServerError> {
    let text = response.text().await?;
    let document = scraper::Html::parse_document(&text);

    let title_selector = Selector::parse("title").expect("Invalid selector for title");
    let title = document
        .select(&title_selector)
        .next()
        .and_then(|t| t.text().next())
        .unwrap_or_default();

    if MAINTENANCE_TITLE == title {
        return Err(TibiaClientError::Maintenance)?;
    };

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
                killed_players: kp_day.parse().context("Failed to parse killed_players")?,
                killed_by_players: kbp_day
                    .parse()
                    .context("Failed to parse killed_by_players")?,
            };

            stats.total_last_week = KilledAmounts {
                killed_players: kp_week.parse().context("Failed to parse killed_players")?,
                killed_by_players: kbp_week
                    .parse()
                    .context("Failed to parse killed_by_players")?,
            };
            continue;
        }

        let last_day = KilledAmounts {
            killed_players: kp_day.parse().context("Failed to parse killed_players")?,
            killed_by_players: kbp_day
                .parse()
                .context("Failed to parse killed_by_players")?,
        };

        let last_week = KilledAmounts {
            killed_players: kp_week.parse().context("Failed to parse killed_players")?,
            killed_by_players: kbp_week
                .parse()
                .context("Failed to parse killed_by_players")?,
        };

        stats.races.push(RaceKillStatistics {
            race: name.to_string(),
            last_day,
            last_week,
        })
    }

    Ok(Some(stats))
}
