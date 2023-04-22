use std::collections::HashMap;

use axum::{
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Json, Router,
};
use utoipa::{openapi::InfoBuilder, OpenApi};
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

#[tokio::main]
async fn main() {
    #[derive(OpenApi)]
    #[openapi(
        paths(
            v1::worlds::list_worlds,
            v1::worlds::get_world_kill_statistics,
            v1::worlds::get_world_details,
            v1::worlds::get_world_guilds
        ),
        components(schemas(
            ApiError,
            tibia_api::WorldsData,
            tibia_api::World,
            tibia_api::GameWorldType,
            tibia_api::TransferType,
            tibia_api::Location,
            tibia_api::PvpType,
            tibia_api::MonsterStats,
            tibia_api::KillStatistics,
            tibia_api::MonsterStats,
            tibia_api::Vocation,
            tibia_api::Player,
            tibia_api::WorldDetails,
            tibia_api::Guild
        )),
        tags((name = "Worlds", description = "World related endpoints"))
    )]
    struct ApiDocV1;
    let mut openapi = ApiDocV1::openapi();
    openapi.info = InfoBuilder::new()
        .title("Tibia API")
        .description(Some(API_DESCRIPTION))
        .version("1.0.0")
        .build();

    let state = AppState::new();

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", openapi))
        .route("/api-docs", get(redocly))
        .route("/__healthcheck", get(healthcheck))
        .route("/favicon.png", get(favicon))
        .route("/", get(redirect_to_swagger_ui))
        .route("/api/v1/worlds", get(v1::worlds::list_worlds))
        .route(
            "/api/v1/worlds/:world_name",
            get(v1::worlds::get_world_details),
        )
        .route(
            "/api/v1/worlds/:world_name/guilds",
            get(v1::worlds::get_world_guilds),
        )
        .route(
            "/api/v1/worlds/:world_name/kill-statistics",
            get(v1::worlds::get_world_kill_statistics),
        );

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

async fn redocly() -> Html<String> {
    let html = REDOCLY_HTML
        .replace("{title}", "Tibia API")
        .replace("{spec_url}", "/api-docs/openapi.json");
    Html::from(html)
}

async fn favicon() -> &'static [u8] {
    include_bytes!("../static/favicon.png")
}

const API_DESCRIPTION: &'static str = r#"
<div style="display: flex; align-items: center; gap: 2rem;">
<img src="/favicon.png" alt="Sorcerer asset" width="150" height="150">
<h1 style="margin: 0; font-size: 2.5rem;">Tibia API</h1>
</div>

This is a helper API for grabbing the data available on the [Tibia](https://www.tibia.com/) website, written in [Rust](https://www.rust-lang.org/). It is primarily a way for me to test out Rust and its ecosystem, but feel free to use it.

The source code is available on [GitHub](https://github.com/ankarhem/tibia-api-rust).

Contact me at [jakob@ankarhem.dev](mailto:jakob@ankarhem.dev), or raise an [issue](https://github.com/ankarhem/tibia-api-rust/issues).

<h2>Disclaimer</h2>

The data is based on [tibia.com](https://www.tibia.com/), the only official Tibia website.

Tibia is a registered trademark of [CipSoft GmbH](https://www.cipsoft.com/en/). Tibia and all products related to Tibia are copyright by [CipSoft GmbH](https://www.cipsoft.com/en/).
"#;

const REDOCLY_HTML: &'static str = r#"
<!DOCTYPE html>
<html>
  <head>
    <title>{title}</title>
    <!-- needed for adaptive design -->
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <link rel="icon" href="/favicon.png">
    <link
      href="https://fonts.googleapis.com/css?family=Montserrat:300,400,700|Roboto:300,400,700"
      rel="stylesheet"
    />

    <!--
    Redoc doesn't change outer page styles
    -->
    <style>
      body {
        margin: 0;
        padding: 0;
      }
    </style>
  </head>
  <body>
    <!--
    Redoc element with link to your OpenAPI definition
    -->
    <redoc spec-url="{spec_url}"></redoc>
    <!--
    Link to Redoc JavaScript on CDN for rendering standalone element
    -->
    <script src="https://cdn.redoc.ly/redoc/latest/bundles/redoc.standalone.js"></script>
  </body>
</html>
"#;
