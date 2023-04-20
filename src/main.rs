use std::collections::HashMap;

use axum::{
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Json, Router,
};
use utoipa::{
    openapi::{ContactBuilder, InfoBuilder},
    OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

mod handlers;
pub use crate::handlers::{v1, ApiError};

#[derive(Clone)]
pub struct AppState {
    client: ClientWithMiddleware,
}

impl AppState {
    fn new() -> AppState {
        // user agent + encoding is required otherwise they block the request
        // encoding is added automatically due to optional features gzip, deflate and brotli enabled
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/113.0")
            .brotli(true)
            .deflate(true)
            .gzip(true)
            .build()
            .expect("Client::builder()");

        let client = ClientBuilder::new(client)
            .with(Cache(HttpCache {
                mode: CacheMode::Default,
                manager: CACacheManager::default(),
                options: None,
            }))
            .build();
        AppState { client }
    }
}

const API_DESCRIPTION: &'static str = r#"
The data is based on [tibia.com](https://www.tibia.com), the only official Tibia website.

Tibia is a registered trademark of [CipSoft GmbH](https://www.cipsoft.com/en/). Tibia and all products related to Tibia are copyright by [CipSoft GmbH](https://www.cipsoft.com/en/).
"#;

#[tokio::main]
async fn main() {
    #[derive(OpenApi)]
    #[openapi(
        paths(v1::worlds::list_worlds, v1::worlds::get_kill_statistics,),
        components(schemas(
            ApiError,
            tibia_api::WorldsData,
            tibia_api::World,
            tibia_api::WorldTag,
            tibia_api::PvpType,
            tibia_api::MonsterStats,
            tibia_api::KillStatistics,
        )),
        tags((name = "Worlds", description = "World related endpoints"))
    )]
    struct ApiDocV1;
    let mut openapi = ApiDocV1::openapi();
    openapi.info = InfoBuilder::new()
        .title("Tibia API")
        .description(Some(API_DESCRIPTION))
        .version("1.0.0")
        .contact(Some(
            ContactBuilder::new()
                .name(Some("Jakob Ankarhem"))
                .email(Some("jakob@ankarhem.dev"))
                .url(Some("https://github.com/ankarhem"))
                .build(),
        ))
        .build();

    let state = AppState::new();

    let app = Router::new()
        .merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", openapi))
        .route("/", get(redirect_to_swagger_ui))
        .route("/__healthcheck", get(healthcheck))
        .route(
            "/api/v1/worlds/:world_name/kill-statistics",
            get(v1::worlds::get_kill_statistics),
        )
        .route("/api/v1/worlds", get(v1::worlds::list_worlds));

    let server = axum::Server::bind(&"0.0.0.0:7032".parse().unwrap())
        .serve(app.with_state(state).into_make_service());
    let addr = server.local_addr();

    println!("Listening on {addr}");

    server.await.unwrap();
}

async fn redirect_to_swagger_ui() -> Redirect {
    Redirect::temporary("/api-docs")
}

async fn healthcheck() -> Response {
    let mut resp = HashMap::new();
    resp.insert("ok", true);
    Json(resp).into_response()
}
