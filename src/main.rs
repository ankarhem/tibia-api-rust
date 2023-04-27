use std::collections::HashMap;

pub use self::error::{Result, ServerError};
pub use self::tibia_page::TibiaPage;

use axum::{
    extract::FromRef,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use tower_http::services::ServeDir;

use utoipa_swagger_ui::SwaggerUi;

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

mod error;
mod handlers;
mod tibia_page;
mod utils;
use crate::handlers::v1;
use utils::{openapi, redocly};

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
    let openapi_docs = openapi::create_openapi_docs();

    let state = AppState::new();
    let static_service = ServeDir::new("static");
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", openapi_docs))
        .route("/api-docs", get(redocly::redocly_index))
        .route("/__healthcheck", get(healthcheck))
        .route("/", get(redocly::redirect_redocly))
        .nest("/api", v1::router(state.clone()))
        .layer(axum::middleware::map_response(main_response_mapper))
        .fallback_service(static_service);

    let addr = "0.0.0.0:7032".parse().unwrap();
    println!("Listening on {}", addr);
    let server = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install CTRL+C signal handler");
        });

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

async fn healthcheck() -> impl IntoResponse {
    let mut resp = HashMap::new();
    resp.insert("ok", true);
    Json(resp)
}
