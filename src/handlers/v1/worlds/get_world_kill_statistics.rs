use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    Json,
};
use scraper::Selector;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{AppState, Result, ServerError, TibiaPage};

use super::{PathParams, COMMUNITY_URL};

#[derive(Debug, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct KillStatistics {
    killed_players: u32,
    killed_by_players: u32,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonsterKillStatistics {
    name: String,
    last_day: KillStatistics,
    last_week: KillStatistics,
}

/// List killstatistics for a world.
///
#[utoipa::path(
    get,
    path = "/api/v1/worlds/{world_name}/kill-statistics",
    params(
        ("world_name" = String, Path, description = "World name", example = "Antica")
    ),
    responses(
        (status = 200, description = "Success", body = [MonsterKillStatistics]),
        (status = 404, description = "Not Found", body = ClientError),
        (status = 500, description = "Internal Server Error", body = ClientError),
    ),
    tag = "Worlds"
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<AppState>,
    Path(path_params): Path<PathParams>,
) -> Result<Json<Vec<MonsterKillStatistics>>> {
    let client = state.client;

    // Form data
    let mut params = HashMap::new();
    params.insert("subtopic", "killstatistics");
    params.insert("world", &path_params.world_name);

    let response = client
        .get(COMMUNITY_URL)
        .query(&params)
        .send()
        .await
        .map_err(|_| ServerError::RequestFail)?;

    let page_as_str = response
        .text()
        .await
        .map_err(|_| ServerError::RequestDecodeBodyFail)?;

    let tibia_page = TibiaPage::new(&page_as_str);
    let main_content = tibia_page.get_main_content()?;

    let table_cell_selector = Selector::parse("#KillStatisticsTable tr.DataRow > td")
        .expect("Invalid selector for kill statistics table");

    let cells = main_content
        .select(&table_cell_selector)
        .map(|cell| cell.inner_html())
        .collect::<Vec<String>>();

    if cells.len() == 0 {
        return Err(ServerError::ScrapeIs404Page);
    }

    let mut iter = cells.iter();

    let mut stats: Vec<MonsterKillStatistics> = vec![];
    while let (Some(name), Some(kp_day), Some(kbp_day), Some(kp_week), Some(kbp_week)) = (
        iter.next(),
        iter.next(),
        iter.next(),
        iter.next(),
        iter.next(),
    ) {
        let last_day = KillStatistics {
            killed_players: kp_day.parse()?,
            killed_by_players: kbp_day.parse()?,
        };

        let last_week = KillStatistics {
            killed_players: kp_week.parse()?,
            killed_by_players: kbp_week.parse()?,
        };

        stats.push(MonsterKillStatistics {
            name: name.to_string(),
            last_day,
            last_week,
        })
    }

    Ok(Json(stats))
}
