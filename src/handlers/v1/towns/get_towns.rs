use std::collections::HashMap;

use axum::{extract::State, Json};
use scraper::Selector;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    prelude::COMMUNITY_URL, tibia_page::sanitize_string, AppState, Result, ServerError, TibiaPage,
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Town(String);

/// List all towns that have houses.
///
#[utoipa::path(
    get,
    operation_id = "get_towns",
    path = "/api/v1/towns",
    responses(
        (status = 200, description = "Success", body = [String], example = json!([
            "Ab\'Dendriel",
            "Ankrahmun",
            "Carlin",
            "Darashia",
            "Edron",
            "Farmine",
            "Gray Beach",
            "Issavi",
            "Kazordoon",
            "Liberty Bay",
            "Moonfall",
            "Port Hope",
            "Rathleton",
            "Silvertides",
            "Svargrond",
            "Thais",
            "Venore",
            "Yalahar",
        ])),
        (status = 500, description = "Internal Server Error", body = ClientError),
    ),
    tag = "Towns"
)]
#[axum::debug_handler]
pub async fn handler(State(state): State<AppState>) -> Result<Json<Vec<Town>>> {
    let client = state.client;

    let mut params = HashMap::new();
    params.insert("subtopic", "houses");
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
        Selector::parse("#houses table.TableContent").expect("Invalid selector for towns table");
    let table = main_content
        .select(&tables_selector)
        .last()
        .ok_or(ServerError::ScrapeUnexpectedPageContent)?;

    let towns_row_selector =
        Selector::parse("tr > td[valign=\"top\"").expect("Invalid selector for towns row");
    let towns_row = table
        .select(&towns_row_selector)
        .next()
        .ok_or(ServerError::ScrapeUnexpectedPageContent)?;

    let mut towns: Vec<Town> = vec![];

    let town_selector = Selector::parse("label").expect("Invalid selector for town");
    for town in towns_row.select(&town_selector) {
        let town_name = town.text().collect::<String>();

        if !town_name.is_empty() {
            let sanitized = sanitize_string(&town_name);
            towns.push(Town(sanitized));
        }
    }

    Ok(Json(towns))
}

#[cfg(test)]
mod tests {
    use crate::tests::get_path;
    use serde_json::{json, Value};

    #[tokio::test]
    async fn it_can_parse_towns() {
        let response = get_path("/api/v1/towns").await;
        assert_eq!(response.status(), 200);

        let received_json = response.json::<Value>().await.unwrap();
        let expected_json = json!([
            "Ab\'Dendriel",
            "Ankrahmun",
            "Carlin",
            "Darashia",
            "Edron",
            "Farmine",
            "Gray Beach",
            "Issavi",
            "Kazordoon",
            "Liberty Bay",
            "Moonfall",
            "Port Hope",
            "Rathleton",
            "Silvertides",
            "Svargrond",
            "Thais",
            "Venore",
            "Yalahar",
        ]);
        assert_eq!(received_json, expected_json);
    }
}
