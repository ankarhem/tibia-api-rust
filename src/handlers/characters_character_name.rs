use std::collections::HashMap;

use crate::models::{CharacterInfo, GuildMember, House, Sex};
use crate::{prelude::*, AppState};
use anyhow::{anyhow, Context, Result};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};

use chrono::{prelude::*, TimeZone, Utc};
use chrono_tz::Europe::Stockholm;
use itertools::Itertools;
use regex::Regex;
use reqwest::{Response, StatusCode};
use reqwest_middleware::ClientWithMiddleware;
use scraper::Selector;
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Serialize, Deserialize, Debug, utoipa::IntoParams)]
pub struct CharacterParams {
    /// Name of world
    #[param(example = "Urinchoklad")]
    pub character_name: String,
}

/// World
///
#[utoipa::path(
    get,
    operation_id = "get_character",
    path = "/api/v1/characters/{character_name}",
    params(CharacterParams),
    responses(
        (status = 200, description = "Success", body = CharacterInfo),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),
        (status = 503, description = "Service Unavailable", body = PublicErrorBody)
    ),
    tag = "Characters"
)]
#[axum::debug_handler]
#[instrument(name = "Get Character", skip(state))]
pub async fn get(
    State(state): State<AppState>,
    Path(path_params): Path<CharacterParams>,
) -> Result<impl IntoResponse, ServerError> {
    let client = &state.client;
    let character_name = path_params.character_name;

    let response = fetch_character_page(client, &character_name)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch character page: {:?}", e);
            e
        })?;
    let character_info = parse_character_page(response, &character_name)
        .await
        .map_err(|e| {
            tracing::error!("Failed to parse character page: {:?}", e);
            e
        })?;

    match character_info {
        Some(c) => Ok(Json(c).into_response()),
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

#[instrument(skip(client))]
pub async fn fetch_character_page(
    client: &ClientWithMiddleware,
    character_name: &str,
) -> Result<Response, reqwest_middleware::Error> {
    let mut params = HashMap::new();
    params.insert("name", character_name);
    let response = client.get(COMMUNITY_URL).query(&params).send().await?;

    Ok(response)
}

#[instrument(skip(response))]
pub async fn parse_character_page(
    response: Response,
    character_name: &str,
) -> Result<Option<CharacterInfo>> {
    let text = response.text().await?;
    let document = scraper::Html::parse_document(&text);

    let selector = Selector::parse(".main-content").expect("Invalid selector for main content");
    let main_content = &document
        .select(&selector)
        .next()
        .context("ElementRef for main content not found")?;

    let table_selector =
        Selector::parse(".TableContainer table.TableContent").expect("Selector to be valid");
    let mut tables = main_content.select(&table_selector);

    // is a 404 page
    if tables.clone().count() == 0 {
        tracing::info!("Character '{}' not found", character_name);
        return Ok(None);
    }

    let row_selector = Selector::parse(".TableContent tr").expect("Selector to be valid");
    let info_rows = tables
        .next()
        .context("Info table not found")?
        .select(&row_selector);

    let cell_selector = Selector::parse("td").expect("Selector to be valid");
    let link_selector = Selector::parse("a").expect("Selector to be valid");

    let mut character = CharacterInfo {
        name: "".to_string(),
        former_names: None,
        title: None,
        sex: Sex::Male,
        vocation: None,
        level: 0,
        achievement_points: 0,
        world: "Antica".to_string(),
        spawn_point: "Yalahar".to_string(),
        houses: None,
        guild: None,
        last_login: None,
        comment: None,
        has_premium: false,
    };

    for row in info_rows {
        println!("{}", row.inner_html());
        let (key, value) = row
            .select(&cell_selector)
            .collect_tuple()
            .context("Expected character info table to contain 2 columns")?;

        let key = key
            .text()
            .next()
            .context("Expected key to contain text")?
            .sanitize();

        match key.as_ref() {
            "Name:" => {
                let value = value.text().next().context("Name not found")?;
                character.name = value.sanitize();
            }
            "Former Names:" => {
                let value: Vec<String> = value
                    .text()
                    .next()
                    .map(|s| s.split(',').map(|s| s.sanitize()).collect())
                    .context("Could not parse former names")?;
                character.former_names = Some(value);
            }
            "Title:" => {
                let value = value.text().next().context("Title not found")?;
                let re = Regex::new(r"(.*) \((\d).*\)").unwrap();
                let captures = re.captures(value);
                let current_title = captures
                    .and_then(|c| c.get(1))
                    .map(|t| t.as_str().sanitize())
                    .context(format!("Could not parse title: {}", value))?;
                character.title = Some(current_title);
            }
            "Sex:" => {
                let value = value.text().next().context("Sex not found")?;
                let sex = match value {
                    "male" => Sex::Male,
                    "female" => Sex::Female,
                    _ => Err(anyhow!(format!("Could not parse sex: {}", value)))?,
                };
                character.sex = sex;
            }
            "Vocation:" => {
                let value = value.text().next().context("Vocation not found")?;
                let vocation = value.parse()?;
                character.vocation = Some(vocation);
            }
            "Level:" => {
                let value = value.text().next().context("Level not found")?;
                let level = value
                    .parse()
                    .context(format!("Could not parse level: {}", value))?;
                character.level = level;
            }
            "Achievement Points:" => {
                let value = value
                    .text()
                    .next()
                    .context("Achievement points not found")?;
                let points = value
                    .parse()
                    .context(format!("Could not parse achievement points: {}", value))?;
                character.achievement_points = points;
            }
            "Residence:" => {
                let value = value.text().next().context("Residence not found")?;
                if value.is_empty() {
                    return Err(anyhow!("Residence is empty"))?;
                }
                character.spawn_point = value.sanitize();
            }
            "World:" => {
                let value = value.text().next().context("World not found")?;
                if value.is_empty() {
                    return Err(anyhow!("World is empty"))?;
                }
                character.world = value.sanitize();
            }
            "House:" => {
                let house_link_el = value
                    .select(&link_selector)
                    .next()
                    .context("Link element not found")?;

                let house_link = house_link_el
                    .value()
                    .attr("href")
                    .context("House link not found")?;

                let re = Regex::new(r"houseid=(\d+)").unwrap();
                let house_id = re
                    .captures(house_link)
                    .and_then(|c| c.get(1))
                    .map(|s| s.as_str().parse::<u32>().unwrap())
                    .context("Could not parse house ID")?;

                let house_name = house_link_el
                    .text()
                    .next()
                    .context("House name not found")?
                    .sanitize();

                let value = value
                    .text()
                    .skip(2)
                    .collect::<Vec<&str>>()
                    .first()
                    .context("House text not found")?
                    .sanitize();
                // (Thais) is paid until Sep 08 2023
                let re = Regex::new(r"\((.*)\) is paid until (.*)").expect("Invalid regex");
                let town = re
                    .captures(&value)
                    .and_then(|c| c.get(1))
                    .map(|t| t.as_str().sanitize())
                    .context(format!("Could not parse town: {}", value))?;
                let paid_until = re
                    .captures(&value)
                    .and_then(|c| c.get(2))
                    .map(|t| t.as_str().sanitize())
                    .context(format!("Could not parse paid until: {}", value))?;

                let naive_date = NaiveDate::parse_from_str(&paid_until, "%B %d %Y")
                    .context(format!("Failed to parse date {}", paid_until))?;

                let house = House {
                    id: house_id,
                    name: house_name,
                    paid_until: naive_date,
                    town,
                };
                if let Some(houses) = &mut character.houses {
                    houses.push(house)
                } else {
                    character.houses = Some(vec![house]);
                }
            }
            "Guild Membership:" => {
                let guild_name = value
                    .select(&link_selector)
                    .next()
                    .and_then(|e| e.text().next())
                    .context("Guild name not found")?
                    .sanitize();

                let value = value.text().next().context("Role not found")?;
                let re = Regex::new(r#"(.*) of the"#).expect("Invalid regex");
                let role = re
                    .captures(value)
                    .and_then(|c| c.get(1))
                    .map(|r| r.as_str().sanitize())
                    .context(format!("Could not parse role: {}", value))?;

                character.guild = Some(GuildMember { role, guild_name })
            }
            "Last Login:" => {
                let value = value
                    .text()
                    .next()
                    .context("Last login value not found")?
                    .sanitize();
                let re = Regex::new(r"(.*) CES?T").unwrap();
                let login_date = re
                    .captures(&value)
                    .and_then(|c| c.get(1))
                    .context(format!("Login date not found in {}", value))?
                    .as_str();

                let naive_dt = NaiveDateTime::parse_from_str(login_date, "%b %d %Y, %H:%M:%S")
                    .context(format!("Failed to parse online record date {}", login_date))?;
                let utc_time = Stockholm
                    .from_local_datetime(&naive_dt)
                    .unwrap()
                    .with_timezone(&Utc);

                character.last_login = Some(utc_time);
            }
            "Account Status:" => {
                let value = value.text().next().context("Premium status not found")?;
                let has_premium = match value {
                    "Premium Account" => true,
                    "Free Account" => false,
                    _ => Err(anyhow!("Unexpected premium status: {}", value))?,
                };
                character.has_premium = has_premium;
            }
            _ => Err(anyhow!("Unexpected key: {}", key))?,
        }
    }

    Ok(Some(character))
}
