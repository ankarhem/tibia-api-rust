use std::net::{SocketAddr, TcpListener};
use std::time::Duration;

use anyhow::Result;
use axum::{body::Body, http::Request, routing::get, Router};
use once_cell::sync::Lazy;
use reqwest::{Client, Method};
use tower_http::{
    classify::StatusInRangeAsFailures,
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tower_request_id::{RequestId, RequestIdLayer};
use tracing::info_span;

mod handlers;
pub mod models;
mod prelude;
pub mod telemetry;
mod utils;

use utils::*;

#[derive(Clone)]
pub struct AppState {
    client: reqwest::Client,
}

fn app() -> Router {
    let openapi_docs = openapi::create_openapi_docs();

    let reqwest_client = create_client().expect("To create reqwest client");

    let app_state = AppState {
        client: reqwest_client,
    };

    let public_service = ServeDir::new("public");

    Router::new()
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
        .with_state(app_state)
        .route("/openapi.json", get(handlers::redocly::serve_openapi))
        .with_state(openapi_docs)
        // Omit these from the logs etc.
        .route("/", get(handlers::redocly::redirect_redocly))
        .route("/api-docs", get(handlers::redocly::serve_redocly))
        .route("/__healthcheck", get(handlers::__healthcheck::get))
        .fallback_service(public_service)
}

pub async fn run(listener: TcpListener) -> Result<()> {
    let addr = listener.local_addr()?;

    tracing::info!("Listening on {}", addr);

    axum::Server::from_tcp(listener)?
        .serve(app().into_make_service())
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install CTRL+C signal handler");
        })
        .await?;

    Ok(())
}

// test helpers
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        telemetry::init_subscriber(subscriber);
    } else {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        telemetry::init_subscriber(subscriber);
    }
});

pub fn spawn_app() -> SocketAddr {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("To bind to random port");
    let addr = listener.local_addr().expect("To get local address");

    tokio::spawn(run(listener));

    addr
}

pub fn create_client() -> Result<Client, reqwest::Error> {
    Client::builder()
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/113.0",
        )
        .brotli(true)
        .deflate(true)
        .gzip(true)
        .pool_idle_timeout(Duration::from_secs(15))
        .pool_max_idle_per_host(10)
        .build()
}
