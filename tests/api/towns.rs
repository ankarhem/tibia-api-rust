use super::*;
use reqwest::StatusCode;
use serde_json::{json, Value};
use tibia_api::clients::TibiaClient;

#[tokio::test]
async fn can_get_towns() {
    let mocked_resp = include_str!("../../tests/mocks/towns-200.html");
    let client = TibiaClient::with_sender(MockSender::new(reqwest::StatusCode::OK, mocked_resp));

    let state = AppState { client };
    let addr = spawn_app(state);

    let response = reqwest::get(format!("http://{addr}/api/v1/towns"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let received_json = response.json::<Value>().await.unwrap();
    let expected_towns = vec![
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
    ];

    for town in expected_towns {
        assert!(received_json.as_array().unwrap().contains(&json!(town)));
    }
}
