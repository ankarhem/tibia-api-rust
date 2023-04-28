use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    Json,
};
use capitalize::Capitalize;
use scraper::Selector;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::prelude::COMMUNITY_URL;
use crate::{AppState, Result, ServerError, TibiaPage};

use super::PathParams;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct KilledAmounts {
    killed_players: u32,
    killed_by_players: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RaceKillStatistics {
    race: String,
    last_day: KilledAmounts,
    last_week: KilledAmounts,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct KillStatistics {
    total_last_day: KilledAmounts,
    total_last_week: KilledAmounts,
    races: Vec<RaceKillStatistics>,
}

/// List killstatistics for a world.
///
#[utoipa::path(
    get,
    operation_id = "get_world_kill_statistics",
    path = "/api/v1/worlds/{world_name}/kill-statistics",
    params(
        ("world_name" = String, Path, description = "World name", example = "Antica")
    ),
    responses(
        (status = 200, description = "Success", body = KillStatistics),
        (status = 404, description = "Not Found", body = ClientError),
        (status = 500, description = "Internal Server Error", body = ClientError),
    ),
    tag = "Worlds"
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<AppState>,
    Path(path_params): Path<PathParams>,
) -> Result<Json<KillStatistics>> {
    let client = state.client;
    let world_name = path_params.world_name.capitalize();

    // Form data
    let mut params = HashMap::new();
    params.insert("subtopic", "killstatistics");
    params.insert("world", &world_name);

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

    if cells.is_empty() {
        return Err(ServerError::ScrapeIs404Page);
    }

    let mut iter = cells.iter();

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
        iter.next(),
        iter.next(),
        iter.next(),
        iter.next(),
        iter.next(),
    ) {
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

    Ok(Json(stats))
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::RaceKillStatistics;
    use crate::tests::get_path;

    #[tokio::test]
    async fn it_can_parse_kill_statistics() {
        let response = get_path("/api/v1/worlds/Antica/kill-statistics").await;
        assert_eq!(response.status(), 200);

        let received_json = response.json::<Value>().await.unwrap();
        let killed_players = received_json
            .get("totalLastDay")
            .unwrap()
            .get("killedPlayers")
            .unwrap();

        assert!(killed_players.as_u64().unwrap() > 0);
        let races_json = received_json.get("races").unwrap();
        assert!(races_json.is_array());
        let races: Vec<RaceKillStatistics> = races_json
            .as_array()
            .unwrap()
            .iter()
            .map(|v| serde_json::from_value(v.clone()).unwrap())
            .collect();

        let total_in_races = races.iter().find(|r| r.race == "Total");
        assert!(total_in_races.is_none());
    }

    #[tokio::test]
    async fn it_can_handle_lowercase() {
        let response = get_path("/api/v1/worlds/antica/kill-statistics").await;
        assert_eq!(response.status(), 200);

        let received_json = response.json::<Value>().await.unwrap();
        let killed_players = received_json
            .get("totalLastDay")
            .unwrap()
            .get("killedPlayers")
            .unwrap();

        assert!(killed_players.as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn it_returns_404_for_invalid_world() {
        let response = get_path("/api/v1/worlds/invalid_world/kill-statistics").await;
        assert_eq!(response.status(), 404);
    }
}
