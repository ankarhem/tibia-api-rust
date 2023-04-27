use std::{collections::HashMap, str::FromStr};

use axum::extract::{Path, State};
use axum::Json;
use capitalize::Capitalize;
use scraper::Selector;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{AppState, Result, ServerError, TibiaPage};

use crate::tibia_page::sanitize_string;

use super::{
    get_worlds::{GameWorldType, Location, PvpType, TransferType},
    PathParams, COMMUNITY_URL,
};

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum Vocation {
    Knight,
    EliteKnight,
    Sorcerer,
    MasterSorcerer,
    Druid,
    ElderDruid,
    Paladin,
    RoyalPaladin,
}

impl FromStr for Vocation {
    type Err = ServerError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "Knight" => Ok(Vocation::Knight),
            "Elite Knight" => Ok(Vocation::EliteKnight),
            "Sorcerer" => Ok(Vocation::Sorcerer),
            "Master Sorcerer" => Ok(Vocation::MasterSorcerer),
            "Druid" => Ok(Vocation::Druid),
            "Elder Druid" => Ok(Vocation::ElderDruid),
            "Paladin" => Ok(Vocation::Paladin),
            "Royal Paladin" => Ok(Vocation::RoyalPaladin),
            _ => Err(ServerError::ScrapeUnexpectedPageContent),
        }
    }
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    #[schema(example = "Urinchoklad")]
    name: String,
    #[schema(example = "52")]
    level: u32,
    vocation: Option<Vocation>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorldDetails {
    #[schema(example = "Antica")]
    name: String,
    is_online: bool,
    #[schema(example = "152")]
    players_online_count: u32,
    #[schema(example = "1211")]
    players_online_record: u32,
    players_online_record_date: String,
    creation_date: String,
    location: Location,
    pvp_type: PvpType,
    #[schema(example = json!(["Rise of Devovorga", "The Lightbearer"]))]
    world_quest_titles: Vec<String>,
    battl_eye: bool,
    battl_eye_date: Option<String>,
    game_world_type: GameWorldType,
    transfer_type: Option<TransferType>,
    premium_required: bool,
    players_online: Vec<Player>,
}

/// Show details for a world.
///
#[utoipa::path(
    get,
    operation_id = "get_world",
    path = "/api/v1/worlds/{world_name}",
    params(
        ("world_name" = String, Path, description = "World name", example = "Antica")
    ),
    responses(
        (status = 200, description = "Success", body = WorldDetails),
        (status = 404, description = "Not Found", body = ClientError),
        (status = 500, description = "Internal Server Error", body = ClientError),
    ),
    tag = "Worlds"
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<AppState>,
    Path(path_params): Path<PathParams>,
) -> Result<Json<WorldDetails>> {
    let client = state.client;
    let world_name = path_params.world_name.capitalize();

    let mut params = HashMap::new();
    params.insert("subtopic", "worlds");
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

    let tables_selector =
        Selector::parse(".InnerTableContainer").expect("Invalid selector for worlds table");
    let mut tables = main_content.select(&tables_selector);

    let table_count = tables.clone().count();
    if table_count == 1 {
        return Err(ServerError::ScrapeIs404Page);
    }

    if let (Some(_), Some(information_table), Some(players_online_table)) =
        (tables.next(), tables.next(), tables.next())
    {
        let cell_selector = Selector::parse("td").expect("Invalid selector for table cell");
        let mut information_cells = information_table.select(&cell_selector);

        let mut world_details = WorldDetails {
            name: world_name,
            is_online: true,
            players_online_count: 0,
            players_online_record: 0,
            players_online_record_date: "".to_string(),
            creation_date: "".to_string(),
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

        while let (Some(header), Some(value)) = (information_cells.next(), information_cells.next())
        {
            match header.inner_html().as_str() {
                "Status:" => {
                    let value = value.text().next().map(|s| s.trim());
                    let status = match value {
                        Some("Online") => true,
                        Some("Offline") => false,
                        _ => Err(ServerError::ScrapeUnexpectedPageContent)?,
                    };
                    world_details.is_online = status;
                }
                "Players Online:" => {
                    let value = value.inner_html().replace(",", "");
                    let players_online_count = value
                        .parse()
                        .map_err(|_| ServerError::ScrapeUnexpectedPageContent)?;
                    world_details.players_online_count = players_online_count;
                }
                "Online Record:" => {
                    let string = value.inner_html();
                    let end_players = string
                        .find(" players")
                        .ok_or(ServerError::ScrapeUnexpectedPageContent)?;

                    let players_string = &string[..end_players].to_string().replace(",", "");

                    let players: u32 = players_string
                        .parse()
                        .map_err(|_| ServerError::ScrapeUnexpectedPageContent)?;
                    world_details.players_online_record = players;

                    let start_date = string
                        .find("(on ")
                        .ok_or(ServerError::ScrapeUnexpectedPageContent)?
                        + 3;
                    let end_date = string
                        .find(")")
                        .ok_or(ServerError::ScrapeUnexpectedPageContent)?;

                    let date_string = sanitize_string(&string[start_date..end_date]);
                    world_details.players_online_record_date = date_string;
                }
                "Creation Date:" => {
                    let date = value.inner_html();
                    world_details.creation_date = date;
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
                        titles.push(title.inner_html());
                    }

                    world_details.world_quest_titles = titles;
                }
                "BattlEye Status:" => {
                    let string = value.inner_html();
                    if string.contains("release") {
                        world_details.battl_eye = true;
                    } else if string.contains("since") {
                        let start = string
                            .find("since ")
                            .ok_or(ServerError::ScrapeUnexpectedPageContent)?
                            + 6;
                        world_details.battl_eye = true;
                        world_details.battl_eye_date =
                            Some(string[start..string.len()].to_string());
                    } else {
                        world_details.battl_eye = false;
                    }
                }
                "Transfer Type:" => match value.inner_html().as_str() {
                    "locked" => {
                        world_details.transfer_type = Some(TransferType::Locked);
                    }
                    "blocked" => {
                        world_details.transfer_type = Some(TransferType::Blocked);
                    }
                    _ => return Err(ServerError::ScrapeUnexpectedPageContent),
                },
                "Premium Type:" => match value.inner_html().as_str() {
                    "premium" => {
                        world_details.premium_required = true;
                    }
                    _ => return Err(ServerError::ScrapeUnexpectedPageContent),
                },
                "Game World Type:" => match value.inner_html().as_str() {
                    "Regular" => {
                        world_details.game_world_type = GameWorldType::Regular;
                    }
                    "Experimental" => {
                        world_details.game_world_type = GameWorldType::Experimental;
                    }
                    _ => return Err(ServerError::ScrapeUnexpectedPageContent),
                },
                _ => return Err(ServerError::ScrapeUnexpectedPageContent),
            }
        }

        let player_cell_selector =
            Selector::parse("tr.Odd > td, tr.Even > td").expect("Invalid selector for player cell");
        let mut player_cells = players_online_table.select(&player_cell_selector);

        while let (Some(name), Some(level), Some(vocation)) = (
            player_cells.next(),
            player_cells.next(),
            player_cells.next(),
        ) {
            let vocation_string = sanitize_string(&vocation.inner_html());
            let vocation = match vocation_string.as_str() {
                "None" => None,
                _ => Some(vocation_string.parse()?),
            };
            let player = Player {
                name: name
                    .text()
                    .next()
                    .ok_or(ServerError::ScrapeUnexpectedPageContent)?
                    .to_string(),
                level: level.inner_html().parse()?,
                vocation,
            };
            world_details.players_online.push(player);
        }

        return Ok(Json(world_details));
    } else {
        return Err(ServerError::ScrapeUnexpectedPageContent);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::tests::get_path;

    #[tokio::test]
    async fn it_can_parse_world_details() {
        let response = get_path("/api/v1/worlds/Antica").await;
        assert_eq!(response.status(), 200);

        let received_json = response.json::<Value>().await.unwrap();
        let name = received_json.get("name").unwrap();
        assert_eq!(name, "Antica");
    }

    #[tokio::test]
    async fn it_can_handle_lowercase() {
        let response = get_path("/api/v1/worlds/antica").await;
        assert_eq!(response.status(), 200);

        let received_json = response.json::<Value>().await.unwrap();
        let name = received_json.get("name").unwrap();
        assert_eq!(name, "Antica");
    }

    #[tokio::test]
    async fn it_returns_404_for_invalid_worlds() {
        let response = get_path("/api/v1/worlds/invalid_world").await;
        assert_eq!(response.status(), 404);
    }
}
