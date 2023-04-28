use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    Json,
};
use capitalize::Capitalize;
use scraper::{ElementRef, Selector};
use serde::Serialize;
use serde_with::skip_serializing_none;
use utoipa::ToSchema;

use crate::prelude::COMMUNITY_URL;
use crate::{AppState, Result, ServerError, TibiaPage};

use super::PathParams;

#[skip_serializing_none]
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Guild {
    logo: Option<String>,
    name: String,
    description: Option<String>,
    active: bool,
}

/// List all guilds for a world.
///
#[utoipa::path(
    get,
    operation_id = "get_world_guilds",
    path = "/api/v1/worlds/{world_name}/guilds",
    params(
        ("world_name" = String, Path, description = "World name", example = "Antica")
    ),
    responses(
        (status = 200, description = "Success", body = [Guild]),
        (status = 404, description = "Not Found", body = ClientError),
        (status = 500, description = "Internal Server Error", body = ClientError),
    ),
    tag = "Worlds"
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<AppState>,
    Path(path_params): Path<PathParams>,
) -> Result<Json<Vec<Guild>>> {
    let client = state.client;
    let world_name = path_params.world_name.capitalize();

    // Form data
    let mut params = HashMap::new();
    params.insert("subtopic", "guilds");
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
    let tables = tibia_page.get_tables()?;
    let tables: Vec<&ElementRef> = tables
        .iter()
        .filter(|t| {
            t.value()
                .has_class("TableContent", scraper::CaseSensitivity::CaseSensitive)
        })
        .collect();

    if tables.len() != 2 {
        return Err(ServerError::ScrapeIs404Page);
    }

    let mut guilds = vec![];

    let row_selector = Selector::parse("tr:not(:first-child)").expect("Invalid selector for rows");
    let cell_selector = Selector::parse("td").expect("Invalid selector for cells");
    let img_selector = Selector::parse("img").expect("Invalid selector for guild logo");

    for i in 0..2 {
        let table = tables
            .first()
            .ok_or(ServerError::ScrapeUnexpectedPageContent)?;

        let rows = table.select(&row_selector);
        for row in rows {
            let mut cells = row.select(&cell_selector);
            if let (Some(logo), Some(name_description), Some(_)) =
                (cells.next(), cells.next(), cells.next())
            {
                let logo = logo
                    .select(&img_selector)
                    .next()
                    .and_then(|img| img.value().attr("src").map(|src| src.to_string()));

                let mut name_description_iterator = name_description.text().take(2);

                let name = name_description_iterator
                    .next()
                    .map(|s| s.to_string())
                    .ok_or(ServerError::ScrapeUnexpectedPageContent)?;

                let description = name_description_iterator.next().map(|s| s.to_string());

                guilds.push(Guild {
                    logo,
                    name,
                    description,
                    active: i == 0,
                });
            } else {
                return Err(ServerError::ScrapeUnexpectedPageContent);
            }
        }
    }

    Ok(Json(guilds))
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::tests::get_path;

    #[tokio::test]
    async fn it_can_parse_world_guilds() {
        let response = get_path("/api/v1/worlds/Antica/guilds").await;
        assert_eq!(response.status(), 200);

        let received_json = response.json::<Value>().await.unwrap();
        assert!(received_json.is_array());
    }

    #[tokio::test]
    async fn it_can_handle_lowercase() {
        let response = get_path("/api/v1/worlds/antica/guilds").await;
        assert_eq!(response.status(), 200);

        let received_json = response.json::<Value>().await.unwrap();
        assert!(received_json.is_array());
    }

    #[tokio::test]
    async fn it_returns_404_for_invalid_world() {
        let response = get_path("/api/v1/worlds/invalid_world/guilds").await;
        assert_eq!(response.status(), 404);
    }
}
