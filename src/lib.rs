use std::str::FromStr;

use anyhow::{anyhow, Result};
use thiserror::Error;

use scraper::Selector;
use serde::Serialize;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KillStatistics {
    killed_players: i32,
    killed_by_players: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonsterStats {
    name: String,
    last_day: KillStatistics,
    last_week: KillStatistics,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid selector: `{0}`")]
    InvalidSelector(String),
    #[error("None value received")]
    NoneValueReceived,
}

pub fn scrape_kill_statistics(page: &str) -> Result<Vec<MonsterStats>> {
    let document = scraper::Html::parse_document(page);

    let selector_str = "#KillStatisticsTable tr.DataRow > td";
    let table_cell_selector = Selector::parse(selector_str)
        .map_err(|_| anyhow!(ParseError::InvalidSelector(selector_str.to_string())))?;
    let cells = document
        .select(&table_cell_selector)
        .map(|cell| cell.inner_html())
        .collect::<Vec<String>>();

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

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
enum WorldTag {
    Blocked,
    Premium,
    Experimental,
    Locked,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
enum PvpType {
    Open,
    Optional,
    Hardcore,
    RetroOpen,
    RetroHardcore,
}

#[derive(Debug)]
struct ParsePvpError;

impl FromStr for PvpType {
    type Err = ParsePvpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let string = s.to_string();
        match string.as_str() {
            "Open PvP" => Ok(PvpType::Open),
            "Optional PvP" => Ok(PvpType::Optional),
            "Hardcore PvP" => Ok(PvpType::Hardcore),
            "Retro Open PvP" => Ok(PvpType::RetroOpen),
            "Retro Hardcore PvP" => Ok(PvpType::RetroHardcore),
            _ => Err(ParsePvpError),
        }
    }
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct World {
    name: String,
    online: i32,
    location: String,
    pvp_type: String,
    battl_eye: bool,
    battl_eye_date: Option<String>,
    tags: Vec<WorldTag>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldsData {
    players_online: i32,
    record_players: i32,
    record_date: String,
    worlds: Vec<World>,
}

pub fn scrape_worlds(page: &str) -> Result<WorldsData> {
    let document = scraper::Html::parse_document(page);

    let tables_selector =
        Selector::parse(".TableContent").map_err(|_| anyhow!("Invalid selector"))?;
    let mut tables = document.select(&tables_selector);

    let mut worlds_data = WorldsData {
        players_online: 0,
        record_players: 0,
        record_date: "".to_string(),
        worlds: vec![],
    };

    if let (Some(record_table), Some(_title), Some(worlds_table)) =
        (tables.next(), tables.next(), tables.next())
    {
        // RECORD PLAYERS
        let record_html = record_table.inner_html();
        let record_date_start = record_html
            .find("(")
            .ok_or(anyhow!("None value received"))?
            + 3; // skip `(on`
        let record_date_end = record_html
            .find(")")
            .ok_or(anyhow!("None value received"))?;
        let record_date = &record_html[record_date_start..record_date_end].replace("&nbsp;", " ");
        let record_date = record_date.trim().to_string();
        worlds_data.record_date = record_date;

        let record_players_start = record_html
            .find("</b>")
            .ok_or(anyhow!("None value received"))?
            + 4; // len
        let record_players_end = record_html
            .find("players")
            .ok_or(anyhow!("None value received"))?;
        let record_players = &record_html[record_players_start..record_players_end]
            .replace("&nbsp;", " ")
            .replace(",", "");
        let record_players = record_players.trim().parse::<i32>()?;
        worlds_data.record_players = record_players;

        // WORLDS
        let world_row_relector = Selector::parse("tr.Odd > td, tr.Even > td")
            .map_err(|_| anyhow!("Invalid selector"))?;
        let name_selector = Selector::parse("a").map_err(|_| anyhow!("Invalid selector"))?;
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
            let mut tags: Vec<WorldTag> = vec![];
            let tags_string = additional_information.inner_html();
            if tags_string.contains("blocked") {
                tags.push(WorldTag::Blocked)
            }
            if tags_string.contains("premium") {
                tags.push(WorldTag::Premium)
            }
            if tags_string.contains("experimental") {
                tags.push(WorldTag::Experimental)
            }
            if tags_string.contains("locked") {
                tags.push(WorldTag::Locked)
            }

            let battl_eye_selector =
                Selector::parse(".HelperDivIndicator").map_err(|_| anyhow!("Invalid selector"))?;

            let world = World {
                name: name
                    .select(&name_selector)
                    .next()
                    .ok_or(anyhow!("Could not parse world name"))?
                    .inner_html(),
                online: online.inner_html().parse()?,
                location: location.inner_html(),
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

                tags,
            };

            worlds_data.worlds.push(world);
        }
    } else {
        return Err(anyhow!("Could not parse world tables"));
    }

    let players_online: i32 = worlds_data.worlds.iter().map(|w| w.online).sum();
    worlds_data.players_online = players_online;

    Ok(worlds_data)
}
