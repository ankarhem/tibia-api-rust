use std::collections::HashMap;

pub use self::error::{Result, ServerError};
pub use self::tibia_page::TibiaPage;

use axum::{
    extract::FromRef,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Json, Router,
};
use tower_http::services::ServeDir;

use utoipa::{openapi::InfoBuilder, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

mod error;
mod handlers;
mod tibia_page;
pub use crate::handlers::v1;

#[derive(Debug, Clone, FromRef)]
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
        servers(
            (url = "https://tibia.ankarhem.dev"),
        ),
        paths(
            v1::worlds::get_worlds::handler,
            v1::worlds::get_world::handler,
            v1::worlds::get_world_kill_statistics::handler,
            v1::worlds::get_world_guilds::handler
        ),
        components(schemas(
            error::ClientErrorCode,
            error::ClientError,
            v1::worlds::get_worlds::WorldsData,
            v1::worlds::get_worlds::World,
            v1::worlds::get_worlds::GameWorldType,
            v1::worlds::get_worlds::TransferType,
            v1::worlds::get_worlds::Location,
            v1::worlds::get_worlds::PvpType,
            v1::worlds::get_world::Player,
            v1::worlds::get_world::Vocation,
            v1::worlds::get_world::WorldDetails,
            v1::worlds::get_world_kill_statistics::MonsterKillStatistics,
            v1::worlds::get_world_kill_statistics::KillStatistics,
            v1::worlds::get_world_guilds::Guild,
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

    let static_service = ServeDir::new("static");

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", openapi))
        .route("/api-docs", get(redocly))
        .route("/__healthcheck", get(healthcheck))
        .route("/", get(redirect_to_swagger_ui))
        .nest("/api", v1::router(state.clone()))
        .layer(axum::middleware::map_response(main_response_mapper))
        .fallback_service(static_service);

    let server =
        axum::Server::bind(&"0.0.0.0:7032".parse().unwrap()).serve(app.into_make_service());
    let addr = server.local_addr();

    println!("Listening on {addr}");

    server.await.unwrap();
}

async fn main_response_mapper(res: Response) -> Response {
    // -- Get the eventual response error.
    let server_error = res.extensions().get::<ServerError>();
    let client_status_error = server_error.map(|se| se.into_client_error());

    // -- If client error, build the new reponse.
    let error_response = client_status_error.map(|ce| ce.into_response());

    if error_response.is_some() {
        println!("Error: {:?}", error_response);
    }

    error_response.unwrap_or(res)
}

async fn redirect_to_swagger_ui() -> Redirect {
    Redirect::temporary("/api-docs")
}

async fn healthcheck() -> impl IntoResponse {
    let mut resp = HashMap::new();
    resp.insert("ok", true);
    Json(resp)
}

async fn redocly() -> impl IntoResponse {
    let html = REDOCLY_HTML
        .replace("{title}", "Tibia API")
        .replace("{spec_url}", "/api-docs/openapi.json");
    Html::from(html)
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
