use std::str::FromStr;

use anyhow::Result;
use thiserror::Error;

use scraper::Selector;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct KillStatistics {
    killed_players: u32,
    killed_by_players: u32,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MonsterStats {
    name: String,
    last_day: KillStatistics,
    last_week: KillStatistics,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Page Not Found")]
    Is404,
    #[error("None value received")]
    UnexpectedPageContent(String),
}

pub fn scrape_kill_statistics(page: &str) -> Result<Vec<MonsterStats>> {
    let document = scraper::Html::parse_document(page);

    let table_cell_selector = Selector::parse("#KillStatisticsTable tr.DataRow > td")
        .expect("Invalid selector for kill statistics table");

    let cells = document
        .select(&table_cell_selector)
        .map(|cell| cell.inner_html())
        .collect::<Vec<String>>();

    if cells.len() == 0 {
        return Err(ParseError::Is404.into());
    }

    let mut iter = cells.iter();

    let mut stats: Vec<MonsterStats> = vec![];
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

        stats.push(MonsterStats {
            name: name.to_string(),
            last_day,
            last_week,
        })
    }

    Ok(stats)
}

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
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let string = s.to_string();
        match string.as_str() {
            "Open PvP" => Ok(PvpType::Open),
            "Optional PvP" => Ok(PvpType::Optional),
            "Hardcore PvP" => Ok(PvpType::Hardcore),
            "Retro Open PvP" => Ok(PvpType::RetroOpen),
            "Retro Hardcore PvP" => Ok(PvpType::RetroHardcore),
            _ => Err(ParseError::UnexpectedPageContent(
                "Unexpected PVP type".into(),
            )),
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
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let string = s.to_string();
        match string.as_str() {
            "Europe" => Ok(Location::Europe),
            "North America" => Ok(Location::NorthAmerica),
            "South America" => Ok(Location::SouthAmerica),
            _ => Err(ParseError::UnexpectedPageContent(
                "Unexpected location".into(),
            )),
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
    Open,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct World {
    name: String,
    online: u32,
    location: Location,
    pvp_type: String,
    battl_eye: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    battl_eye_date: Option<String>,
    premium_required: bool,
    transfer_type: TransferType,
    game_world_type: GameWorldType,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorldsData {
    players_online: u32,
    record_players: u32,
    record_date: String,
    worlds: Vec<World>,
}

pub fn scrape_worlds(page: &str) -> Result<WorldsData> {
    let document = scraper::Html::parse_document(page);

    let tables_selector =
        Selector::parse(".TableContent").expect("Invalid selector for worlds table");
    let mut tables = document.select(&tables_selector);

    let mut worlds_data = WorldsData {
        players_online: 0,
        record_players: 0,
        record_date: "".to_string(),
        worlds: vec![],
    };

    if let (Some(record_table), Some(_), Some(worlds_table)) =
        (tables.next(), tables.next(), tables.next())
    {
        // RECORD PLAYERS
        let record_html = record_table.inner_html();
        let record_date_start = record_html
            .find("(")
            .ok_or(ParseError::UnexpectedPageContent(format!(
                "Unexpected html when trying to parse record players `{record_html}`"
            )))?
            + 3; // skip `(on`
        let record_date_end = record_html
            .find(")")
            .ok_or(ParseError::UnexpectedPageContent(format!(
                "Unexpected html when trying to parse record players `{record_html}`"
            )))?;
        let record_date = &record_html[record_date_start..record_date_end].replace("&nbsp;", " ");
        let record_date = record_date.trim().to_string();
        worlds_data.record_date = record_date;

        let record_players_start =
            record_html
                .find("</b>")
                .ok_or(ParseError::UnexpectedPageContent(format!(
                    "Unexpected html when trying to parse record players `{record_html}`"
                )))?
                + 4; // len
        let record_players_end =
            record_html
                .find("players")
                .ok_or(ParseError::UnexpectedPageContent(format!(
                    "Unexpected html when trying to parse record players `{record_html}`"
                )))?;
        let record_players = &record_html[record_players_start..record_players_end]
            .replace("&nbsp;", " ")
            .replace(",", "");
        let record_players = record_players.trim().parse::<u32>()?;
        worlds_data.record_players = record_players;

        // WORLDS
        let world_row_relector =
            Selector::parse("tr.Odd > td, tr.Even > td").expect("Invalid selector for world row");
        let name_selector = Selector::parse("a").expect("Invalid selector for world name");
        let mut cells = worlds_table.select(&world_row_relector);

        while let (
            Some(name),
            Some(online),
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
                TransferType::Blocked
            } else if additional_information.contains("Locked") {
                TransferType::Locked
            } else {
                TransferType::Open
            };

            let world = World {
                name: name
                    .select(&name_selector)
                    .next()
                    .ok_or(ParseError::UnexpectedPageContent(
                        "Unexpected content when trying to parse name".into(),
                    ))?
                    .inner_html(),
                online: online.inner_html().parse()?,
                location: location.inner_html().parse()?,
                pvp_type: pvp_type.inner_html().parse()?,
                battl_eye: battl_eye.inner_html().len() > 0,
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
                            let date = s[start..end].to_string();

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
        return Err(ParseError::UnexpectedPageContent(format!(
            "Unexpected content when trying to parse worlds page"
        ))
        .into());
    }

    let players_online: u32 = worlds_data.worlds.iter().map(|w| w.online).sum();
    worlds_data.players_online = players_online;

    Ok(worlds_data)
}

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
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Knight" => Ok(Vocation::Knight),
            "Elite Knight" => Ok(Vocation::EliteKnight),
            "Sorcerer" => Ok(Vocation::Sorcerer),
            "Master Sorcerer" => Ok(Vocation::MasterSorcerer),
            "Druid" => Ok(Vocation::Druid),
            "Elder Druid" => Ok(Vocation::ElderDruid),
            "Paladin" => Ok(Vocation::Paladin),
            "Royal Paladin" => Ok(Vocation::RoyalPaladin),
            _ => Err(ParseError::UnexpectedPageContent(format!(
                "Unexpected vocation value `{s}`"
            ))),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    name: String,
    level: u32,
    vocation: Option<Vocation>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorldDetails {
    is_online: bool,
    players_online_count: u32,
    players_online_record: u32,
    players_online_record_date: String,
    creation_date: String,
    location: Location,
    pvp_type: PvpType,
    world_quest_titles: Vec<String>,
    battl_eye: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    battl_eye_date: Option<String>,
    game_world_type: GameWorldType,
    transfer_type: TransferType,
    premium_required: bool,
    players_online: Vec<Player>,
}

pub fn scrape_world_details(page: &str) -> Result<WorldDetails> {
    let document = scraper::Html::parse_document(&page);

    let main_content =
        Selector::parse(".main-content #worlds").expect("Invalid selector for main content");
    let main_content =
        document
            .select(&main_content)
            .next()
            .ok_or(ParseError::UnexpectedPageContent(format!(
                "Unexpected page content when trying to parse main content"
            )))?;

    let tables_selector =
        Selector::parse(".InnerTableContainer").expect("Invalid selector for worlds table");
    let mut tables = main_content.select(&tables_selector);

    let table_count = tables.clone().count();
    if table_count == 1 {
        return Err(ParseError::Is404.into());
    }

    if let (Some(_), Some(information_table), Some(players_online_table)) =
        (tables.next(), tables.next(), tables.next())
    {
        let cell_selector = Selector::parse("td").expect("Invalid selector for table cell");
        let mut information_cells = information_table.select(&cell_selector);

        let mut world_details = WorldDetails {
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
            transfer_type: TransferType::Open,
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
                        Some(s) => {
                            return Err(
                                ParseError::UnexpectedPageContent(format!("Status: `{s}`")).into()
                            )
                        }
                        _ => {
                            return Err(ParseError::UnexpectedPageContent(format!(
                                "Status: `None`"
                            ))
                            .into())
                        }
                    };
                    world_details.is_online = status;
                }
                "Players Online:" => {
                    let value = value.inner_html().replace(",", "");
                    let players_online_count = value.parse().map_err(|_| {
                        ParseError::UnexpectedPageContent(format!("Players Online: `{}`", value))
                    })?;
                    world_details.players_online_count = players_online_count;
                }
                "Online Record:" => {
                    let string = value.inner_html();
                    let end_players =
                        string
                            .find(" players")
                            .ok_or(ParseError::UnexpectedPageContent(format!(
                                "Unexpected text in player record value `{string}`"
                            )))?;

                    let players_string = &string[..end_players].to_string().replace(",", "");

                    let players: u32 = players_string.parse().map_err(|_| {
                        ParseError::UnexpectedPageContent(format!(
                            "Unexpected text in player record value `{players_string}`"
                        ))
                    })?;
                    world_details.players_online_record = players;

                    let start_date =
                        string
                            .find("(on ")
                            .ok_or(ParseError::UnexpectedPageContent(format!(
                                "Unexpected text in player record value `{string}`"
                            )))?
                            + 3;
                    let end_date =
                        string
                            .find(")")
                            .ok_or(ParseError::UnexpectedPageContent(format!(
                                "Unexpected text in player record value `{string}`"
                            )))?;

                    let date_string = unescape_string(&string[start_date..end_date]);
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
                        let start =
                            string
                                .find("since ")
                                .ok_or(ParseError::UnexpectedPageContent(format!(
                                    "Unexpected string in BattlEye value `{string}`"
                                )))?
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
                        world_details.transfer_type = TransferType::Locked;
                    }
                    "blocked" => {
                        world_details.transfer_type = TransferType::Blocked;
                    }
                    other_strings => {
                        return Err(ParseError::UnexpectedPageContent(format!(
                            "Unexpected Transfer Type value `{other_strings}`"
                        ))
                        .into())
                    }
                },
                "Premium Type:" => match value.inner_html().as_str() {
                    "premium" => {
                        world_details.premium_required = true;
                    }
                    other_strings => {
                        return Err(ParseError::UnexpectedPageContent(format!(
                            "Unexpected Premium Type value `{other_strings}`"
                        ))
                        .into())
                    }
                },
                "Game World Type:" => match value.inner_html().as_str() {
                    "Regular" => {
                        world_details.game_world_type = GameWorldType::Regular;
                    }
                    "Experimental" => {
                        world_details.game_world_type = GameWorldType::Experimental;
                    }
                    other_strings => {
                        return Err(ParseError::UnexpectedPageContent(format!(
                            "Unexpected Game World Type `{other_strings}`"
                        ))
                        .into())
                    }
                },
                other_headers => {
                    return Err(ParseError::UnexpectedPageContent(format!(
                        "Unexpected World detail Header `{other_headers}`"
                    ))
                    .into())
                }
            }
        }

        let player_cell_selector =
            Selector::parse("tr.Odd > td, tr.Even > td").expect("Invalid selector for player cell");
        let mut player_cells = players_online_table.select(&player_cell_selector);

        // println!("{}", player_cells.next().unwrap().inner_html());

        while let (Some(name), Some(level), Some(vocation)) = (
            player_cells.next(),
            player_cells.next(),
            player_cells.next(),
        ) {
            let vocation_string = unescape_string(&vocation.inner_html());
            let vocation = match vocation_string.as_str() {
                "None" => None,
                _ => Some(vocation_string.parse()?),
            };
            let player = Player {
                name: name
                    .text()
                    .next()
                    .ok_or(ParseError::UnexpectedPageContent("Player name".into()))?
                    .to_string(),
                level: level.inner_html().parse()?,
                vocation,
            };
            world_details.players_online.push(player);
        }

        return Ok(world_details);
    } else {
        return Err(ParseError::UnexpectedPageContent("Selecting tables".into()).into());
    }
}

fn unescape_string(page: &str) -> String {
    let sanitized = page
        .trim()
        .replace("\\n", "")
        .replace("\\\"", "'")
        .replace("\\u00A0", " ")
        .replace("\\u0026", "&")
        .replace("\\u0026#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&");

    sanitized
}
