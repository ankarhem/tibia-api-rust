use super::*;
use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn can_get_worlds() {
    let body = include_str!("../mocks/worlds-200.html");
    let client = MockedClient::default().body(body);

    let state = AppState::with_client(client);
    let addr = spawn_app(state);

    let response = reqwest::get(format!("http://{addr}/api/v1/worlds"))
        .await
        .unwrap();

    assert_eq!(StatusCode::OK, response.status());

    let received_json = response.json::<Value>().await.unwrap();
    let expected = include_str!("../mocks/worlds-200.json");
    let expected_json = serde_json::from_str::<Value>(expected).unwrap();

    assert_eq!(expected_json, received_json);
}

#[tokio::test]
async fn sends_503_when_maintenance() {
    let body = include_str!("../mocks/maintenance-200.html");
    let client = MockedClient::default().body(body);

    let state = AppState::with_client(client);
    let addr = spawn_app(state);

    let response = reqwest::get(format!("http://{addr}/api/v1/worlds"))
        .await
        .unwrap();

    assert_eq!(StatusCode::SERVICE_UNAVAILABLE, response.status())
}
