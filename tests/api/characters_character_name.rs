use pretty_assertions::assert_eq;
use reqwest::StatusCode;
use tibia_api::*;

#[tokio::test]
async fn can_get_character_info() {
    let addr = spawn_app();

    let characters = ["Rust Api", "Hälge", "bobeek", "Glada Pojken"];

    for character in characters {
        let response = reqwest::get(format!("http://{addr}/api/v1/characters/{character}"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn returns_404_for_invalid_character() {
    let addr = spawn_app();

    let response = reqwest::get(format!("http://{addr}/api/v1/characters/invalid_character"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
