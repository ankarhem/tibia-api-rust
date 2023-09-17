use std::io::Read;

use super::*;
use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn can_get_guilds() {
    let file_path = "tests/mocks/guilds-jaguna-200.html";
    // read contents as bytes
    let mut file = std::fs::File::open(file_path).unwrap();
    let mut buf = vec![];
    file.read_to_end(&mut buf).unwrap();
    // converts iso-8859-1 to utf-8 so we can represent it as a string
    let body = buf.iter().map(|c| *c as char).collect::<String>();

    let client = MockedClient::new().body(&body);

    let state = AppState::with_client(client);
    let addr = spawn_app(state);

    let response = reqwest::get(format!("http://{addr}/api/v1/worlds/Jaguna/guilds"))
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, response.status());

    let received_json = response.json::<Value>().await.unwrap();
    let expected = include_str!("../mocks/guilds-jaguna-200.json");
    let expected_json = serde_json::from_str::<Value>(expected).unwrap();

    assert_eq!(expected_json, received_json);
}

#[tokio::test]
async fn returns_404_for_invalid_world() {
    let body = include_str!("../mocks/guilds-invalid_world-200.html");
    let client = MockedClient::new().body(body);

    let state = AppState::with_client(client);
    let addr = spawn_app(state);

    let response = reqwest::get(format!("http://{addr}/api/v1/worlds/invalid_world/guilds"))
        .await
        .unwrap();
    assert_eq!(StatusCode::NOT_FOUND, response.status());
}

#[tokio::test]
async fn sends_503_when_maintenance() {
    let body = include_str!("../mocks/maintenance-200.html");
    let client = MockedClient::default().body(body);

    let state = AppState::with_client(client);
    let addr = spawn_app(state);

    let response = reqwest::get(format!("http://{addr}/api/v1/worlds/Antica/guilds"))
        .await
        .unwrap();

    assert_eq!(StatusCode::SERVICE_UNAVAILABLE, response.status())
}
