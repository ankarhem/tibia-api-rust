use reqwest::StatusCode;
use serde_json::Value;
use tibia_api::{models::RaceKillStatistics, *};

#[tokio::test]
async fn can_get_guilds() {
    let addr = spawn_app();

    let response = reqwest::get(format!(
        "http://{addr}/api/v1/worlds/Antica/kill-statistics"
    ))
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let received_json = response.json::<Value>().await.unwrap();
    let killed_players = received_json
        .get("totalLastDay")
        .unwrap()
        .get("killedPlayers")
        .unwrap();

    assert!(killed_players.as_u64().unwrap() > 0);
    let races_json = received_json.get("races").unwrap();
    assert!(races_json.is_array());
    let races: Vec<RaceKillStatistics> = races_json
        .as_array()
        .unwrap()
        .iter()
        .map(|v| serde_json::from_value(v.clone()).unwrap())
        .collect();

    let total_in_races = races.iter().find(|r| r.race == "Total");
    assert!(total_in_races.is_none());
}

#[tokio::test]
async fn can_handle_lowercase() {
    let addr = spawn_app();

    let response = reqwest::get(format!(
        "http://{addr}/api/v1/worlds/antica/kill-statistics"
    ))
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let received_json = response.json::<Value>().await.unwrap();
    let killed_players = received_json
        .get("totalLastDay")
        .unwrap()
        .get("killedPlayers")
        .unwrap();

    assert!(killed_players.as_u64().unwrap() > 0);
}

#[tokio::test]
async fn returns_404_for_invalid_world() {
    let addr = spawn_app();

    let response = reqwest::get(format!(
        "http://{addr}/api/v1/worlds/invalid_world/kill-statistics"
    ))
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
