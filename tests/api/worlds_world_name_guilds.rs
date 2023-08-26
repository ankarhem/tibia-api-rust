use reqwest::StatusCode;
use serde_json::Value;
use tibia_api::*;

#[tokio::test]
async fn can_get_guilds() {
    let addr = spawn_app();

    let response = reqwest::get(format!("http://{addr}/api/v1/worlds/Antica/guilds"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let received_json = response.json::<Value>().await.unwrap();
    assert!(received_json.is_array());
}

#[tokio::test]
async fn can_handle_lowercase() {
    let addr = spawn_app();

    let response = reqwest::get(format!("http://{addr}/api/v1/worlds/antica/guilds"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let received_json = response.json::<Value>().await.unwrap();
    assert!(received_json.is_array());
}

#[tokio::test]
async fn returns_404_for_invalid_world() {
    let addr = spawn_app();

    let response = reqwest::get(format!("http://{addr}/api/v1/worlds/invalid_world/guilds"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}