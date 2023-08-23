use std::collections::HashMap;
use std::str::FromStr;

use axum::extract::State;
use axum::Json;
use scraper::Selector;
use serde::Serialize;
use utoipa::ToSchema;

use crate::utils::time::TibiaTime;
use crate::{AppState, ServerError};
use crate::{Result, TibiaPage};

use crate::prelude::COMMUNITY_URL;

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum PvpType {
    Open,
    Optional,
    Hardcore,
    RetroOpen,
    RetroHardcore,
}

#[derive(Debug)]
pub struct ParsePvpError;

impl FromStr for PvpType {
    type Err = ServerError;

    fn from_str(s: &str) -> Result<Self> {
        let string = s.to_string();
        match string.as_str() {
            "Open PvP" => Ok(PvpType::Open),
            "Optional PvP" => Ok(PvpType::Optional),
            "Hardcore PvP" => Ok(PvpType::Hardcore),
            "Retro Open PvP" => Ok(PvpType::RetroOpen),
            "Retro Hardcore PvP" => Ok(PvpType::RetroHardcore),
            _ => Err(ServerError::ScrapeUnexpectedPageContent),
        }
    }
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum Location {
    Europe,
    SouthAmerica,
    NorthAmerica,
}

impl FromStr for Location {
    type Err = ServerError;

    fn from_str(s: &str) -> Result<Self> {
        let string = s.to_string();
        match string.as_str() {
            "Europe" => Ok(Location::Europe),
            "North America" => Ok(Location::NorthAmerica),
            "South America" => Ok(Location::SouthAmerica),
            _ => Err(ServerError::ScrapeUnexpectedPageContent),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum GameWorldType {
    Regular,
    Experimental,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum TransferType {
    Blocked,
    Locked,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct World {
    #[schema(example = "Antica")]
    name: String,
    #[schema(example = "1337")]
    players_online_count: u32,
    location: Location,
    pvp_type: PvpType,
    battl_eye: bool,
    #[schema(example = "2017-08-29")]
    battl_eye_date: Option<TibiaTime>,
    #[schema(example = false)]
    premium_required: bool,
    transfer_type: Option<TransferType>,
    game_world_type: GameWorldType,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorldsData {
    players_online_total: u32,
    record_players: u32,
    #[schema(example = "2007-11-28T18:26:00+00:00")]
    record_date: TibiaTime,
    worlds: Vec<World>,
}

/// List all worlds.
///
#[utoipa::path(
    get,
    operation_id = "get_worlds",
    path = "/api/v1/worlds",
    responses(
        (status = 200, description = "Success", body = WorldsData),
        (status = 500, description = "Internal Server Error", body = ClientError),
    ),
    tag = "Worlds"
)]
#[axum::debug_handler]
pub async fn handler(State(state): State<AppState>) -> Result<Json<WorldsData>> {
    let client = state.client;

    let mut params = HashMap::new();
    params.insert("subtopic", "worlds");
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

    let tables_selector =
        Selector::parse(".TableContent").expect("Invalid selector for worlds table");
    let mut tables = main_content.select(&tables_selector);

    let mut worlds_data = WorldsData {
        players_online_total: 0,
        record_players: 0,
        record_date: TibiaTime::default(),
        worlds: vec![],
    };

    if let (Some(record_table), Some(_), Some(worlds_table)) =
        (tables.next(), tables.next(), tables.next())
    {
        // RECORD PLAYERS
        let record_html = record_table.inner_html();
        let record_date_start = record_html
            .find('(')
            .ok_or(ServerError::ScrapeUnexpectedPageContent)?
            + 3; // skip `(on`
        let record_date_end = record_html
            .find(')')
            .ok_or(ServerError::ScrapeUnexpectedPageContent)?;
        let record_date = &record_html[record_date_start..record_date_end].replace("&nbsp;", " ");
        let record_date = record_date.trim().to_string();
        worlds_data.record_date = record_date
            .parse::<TibiaTime>()
            .map_err(|_| ServerError::ScrapeUnexpectedPageContent)?;

        let record_players_start = record_html
            .find("</b>")
            .ok_or(ServerError::ScrapeUnexpectedPageContent)?
            + 4; // len
        let record_players_end = record_html
            .find("players")
            .ok_or(ServerError::ScrapeUnexpectedPageContent)?;
        let record_players = &record_html[record_players_start..record_players_end]
            .replace("&nbsp;", " ")
            .replace(',', "");
        let record_players = record_players.trim().parse::<u32>()?;
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

            let game_world_type = if additional_information.contains("experimental") {
                GameWorldType::Experimental
            } else {
                GameWorldType::Regular
            };

            let premium_required = additional_information.contains("premium");
            let transfer_type = if additional_information.contains("blocked") {
                Some(TransferType::Blocked)
            } else if additional_information.contains("Locked") {
                Some(TransferType::Locked)
            } else {
                None
            };

            let world = World {
                name: name
                    .select(&name_selector)
                    .next()
                    .ok_or(ServerError::ScrapeUnexpectedPageContent)?
                    .inner_html(),
                players_online_count: players_online.inner_html().replace(",", "").parse()?,
                location: location.inner_html().parse()?,
                pvp_type: pvp_type.inner_html().parse().unwrap(),
                battl_eye: !battl_eye.inner_html().is_empty(),
                battl_eye_date: battl_eye
                    .select(&battl_eye_selector)
                    .next()
                    .and_then(|indic| {
                        indic.value().attr("onmouseover").map(|s| {
                            if s.contains("release") {
                                return None;
                            }

                            let start_pattern = "since ";
                            let end_pattern = ".</p>";
                            let start = s.find(start_pattern).map(|i| i + start_pattern.len())?;
                            let end = s.find(end_pattern)?;
                            let date = s[start..end].to_string().parse::<TibiaTime>().ok()?;

                            Some(date)
                        })
                    })
                    .flatten(),
                premium_required,
                game_world_type,
                transfer_type,
            };

            worlds_data.worlds.push(world);
        }
    } else {
        return Err(ServerError::ScrapeUnexpectedPageContent);
    }

    let players_online_total: u32 = worlds_data
        .worlds
        .iter()
        .map(|w| w.players_online_count)
        .sum();
    worlds_data.players_online_total = players_online_total;

    Ok(Json(worlds_data))
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::tests::get_path;

    #[tokio::test]
    async fn it_can_parse_worlds() {
        let response = get_path("/api/v1/worlds").await;
        assert_eq!(response.status(), 200);

        let received_json = response.json::<Value>().await.unwrap();
        let record_players = received_json.get("recordPlayers").unwrap();

        assert_eq!(record_players, 64_028);
    }
}
