use super::*;
use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn can_get_residences() {
    let body = include_str!("../mocks/houses-jaguna-edron-200.html");
    let client = MockedClient::new().body(body);

    let state = AppState::with_client(client);
    let addr = spawn_app(state);

    let response = reqwest::get(format!(
        "http://{addr}/api/v1/worlds/Jaguna/residences?town=Edron&type=house"
    ))
    .await
    .unwrap();
    assert_eq!(StatusCode::OK, response.status());

    let received_json = response.json::<Value>().await.unwrap();
    let expected = include_str!("../mocks/houses-jaguna-edron-200.json");
    let expected_json = serde_json::from_str::<Value>(expected).unwrap();

    // strip expiryTime from received_json because it's determined at runtime
    let expected_json = expected_json
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            let mut value = v.as_object().unwrap().clone();
            let mut status = value.get("status").unwrap().as_object().unwrap().clone();
            status.remove("expiryTime");

            value.remove("status");
            value.insert("status".to_string(), Value::Object(status));

            Value::Object(value.clone())
        })
        .collect::<Vec<_>>();
    let received_json = received_json
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            let mut value = v.as_object().unwrap().clone();
            let mut status = value.get("status").unwrap().as_object().unwrap().clone();
            status.remove("expiryTime");

            value.remove("status");
            value.insert("status".to_string(), Value::Object(status));

            Value::Object(value.clone())
        })
        .collect::<Vec<_>>();

    assert_eq!(expected_json, received_json);
}

#[tokio::test]
async fn returns_404_for_invalid_world() {
    let body = include_str!("../mocks/houses-invalid_world-edron-200.html");
    let client = MockedClient::new().body(body);

    let state = AppState::with_client(client);
    let addr = spawn_app(state);

    let response = reqwest::get(format!(
        "http://{addr}/api/v1/worlds/invalid_world/residences?town=Edron&type=house"
    ))
    .await
    .unwrap();
    assert_eq!(StatusCode::NOT_FOUND, response.status());
}

#[tokio::test]
async fn returns_404_for_invalid_town() {
    let body = include_str!("../mocks/houses-jaguna-invalid_town-200.html");
    let client = MockedClient::new().body(body);

    let state = AppState::with_client(client);
    let addr = spawn_app(state);

    let response = reqwest::get(format!(
        "http://{addr}/api/v1/worlds/Jaguna/residences?town=invalid_town&type=house"
    ))
    .await
    .unwrap();
    assert_eq!(StatusCode::NOT_FOUND, response.status());
}
