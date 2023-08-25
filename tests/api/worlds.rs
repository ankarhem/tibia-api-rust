use reqwest::StatusCode;
use serde_json::{json, Value};
use tibia_api::*;

#[tokio::test]
async fn can_get_worlds() {
    let addr = spawn_app();

    let response = reqwest::get(format!("http://{addr}/api/v1/worlds"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let received_json = response.json::<Value>().await.unwrap();

    assert_eq!(received_json.get("recordPlayers").unwrap(), 64_028);
    assert_eq!(
        received_json.get("recordDate").unwrap(),
        "2007-11-28T18:26:00Z",
    );
}
