use reqwest::StatusCode;
use serde_json::Value;
use tibia_api::*;

#[tokio::test]
async fn can_get_a_world() {
    let addr = spawn_app();

    let response = reqwest::get(format!("http://{addr}/api/v1/world/Antica"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let received_json = response.json::<Value>().await.unwrap();
    assert_eq!(received_json.get("name").unwrap(), "Antica");
}

#[tokio::test]
async fn can_handle_lowercase() {
    let addr = spawn_app();

    let response = reqwest::get(format!("http://{addr}/api/v1/world/antica"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let received_json = response.json::<Value>().await.unwrap();
    assert_eq!(received_json.get("name").unwrap(), "Antica");
}

#[tokio::test]
async fn returns_404_for_invalid_world() {
    let addr = spawn_app();

    let response = reqwest::get(format!("http://{addr}/api/v1/world/invalid_world"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
