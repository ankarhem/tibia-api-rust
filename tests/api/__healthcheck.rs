use super::*;
use pretty_assertions::assert_eq;
use reqwest::StatusCode;

#[tokio::test]
async fn healthcheck_works() {
    let addr = spawn_app(AppState::default());

    let response = reqwest::get(format!("http://{addr}/__healthcheck"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
