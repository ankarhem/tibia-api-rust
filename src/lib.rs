use std::{
    net::TcpListener,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use axum::{body::Body, http::Request, routing::get, Router};
use clients::Client;
use prelude::TibiaClient;
use reqwest::Method;
use tower_http::{
    classify::StatusInRangeAsFailures,
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tower_request_id::{RequestId, RequestIdLayer};
use tracing::info_span;

pub mod clients;
mod handlers;
pub mod models;
mod prelude;
pub mod telemetry;
mod utils;

use utils::*;

#[derive(Clone)]
pub struct AppState<S: Client> {
    client: S,
    towns: Arc<Mutex<Vec<String>>>,
}

impl AppState<TibiaClient> {
    pub fn with_client<S: Client>(client: S) -> AppState<S> {
        AppState {
            client,
            towns: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl Default for AppState<TibiaClient> {
    fn default() -> Self {
        Self {
            client: TibiaClient::default(),
            towns: Arc::new(Mutex::new(vec![])),
        }
    }
}

pub fn app<C: Client>(state: AppState<C>) -> Router {
    let openapi_docs = openapi::create_openapi_docs();

    let public_service = ServeDir::new("public");

    let app = Router::new()
        .route("/api/v1/towns", get(handlers::towns::get))
        .route("/api/v1/worlds", get(handlers::worlds::get))
        .route(
            "/api/v1/worlds/:world_name",
            get(handlers::worlds_world_name::get),
        )
        .route(
            "/api/v1/worlds/:world_name/guilds",
            get(handlers::worlds_world_name_guilds::get),
        )
        .route(
            "/api/v1/worlds/:world_name/kill-statistics",
            get(handlers::worlds_world_name_kill_statistics::get),
        )
        .route(
            "/api/v1/worlds/:world_name/residences",
            get(handlers::worlds_world_name_residences::get),
        )
        .route("/", get(handlers::redocly::redirect_redocly))
        .route("/api-docs", get(handlers::redocly::serve_redocly))
        .route("/__healthcheck", get(handlers::__healthcheck::get))
        .fallback_service(public_service)
        .with_state(state);

    app.route("/openapi.json", get(handlers::redocly::serve_openapi))
        .with_state(openapi_docs)
        .layer(CompressionLayer::new())
        .layer(
            CorsLayer::new()
                // allow `GET` and `POST` when accessing the resource
                .allow_methods([Method::GET])
                // allow requests from any origin
                .allow_origin(Any),
        )
        .layer(
            TraceLayer::new(StatusInRangeAsFailures::new(400..=599).into_make_classifier())
                // Let's create a tracing span for each request
                .make_span_with(|request: &Request<Body>| {
                    // We get the request id from the extensions
                    let request_id = request
                        .extensions()
                        .get::<RequestId>()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "unknown".into());
                    // And then we put it along with other information into the `request` span
                    info_span!(
                        "request",
                        id = %request_id,
                        method = %request.method(),
                        uri = %request.uri(),
                    )
                })
                .on_response(
                    tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO),
                ),
        )
        .layer(RequestIdLayer)
}

pub async fn run(app: Router, listener: TcpListener) -> Result<()> {
    let addr = listener.local_addr()?;

    tracing::info!("Listening on {}", addr);

    let server = axum::Server::from_tcp(listener)?
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install CTRL+C signal handler");
        });

    // Fills state with towns
    tokio::spawn(async move {
        let _ = reqwest::get(format!("http://{addr}/api/v1/towns")).await;
    });

    server.await?;

    Ok(())
}
