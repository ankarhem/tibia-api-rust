#![feature(async_fn_in_trait)]

use http::response;
use once_cell::sync::Lazy;
use tibia_api::{app, clients::HttpSend, run, telemetry, AppState};

mod __healthcheck;
mod towns;
mod worlds;
mod worlds_world_name;
mod worlds_world_name_guilds;
mod worlds_world_name_kill_statistics;
mod worlds_world_name_residences;

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

pub fn spawn_app<S: HttpSend>(state: AppState<S>) -> std::net::SocketAddr {
    Lazy::force(&TRACING);

    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("To bind to random port");
    let addr = listener.local_addr().expect("To get local address");

    let app = app(state);
    tokio::spawn(run(app, listener));

    addr
}

#[derive(Default, Clone)]
pub struct MockSender {
    status: reqwest::StatusCode,
    body: &'static str,
}

impl MockSender {
    pub fn new(status: reqwest::StatusCode, body: &'static str) -> Self {
        Self { status, body }
    }

    pub fn status(self, status: reqwest::StatusCode) -> Self {
        Self { status, ..self }
    }

    pub fn body(self, body: &'static str) -> Self {
        Self { body, ..self }
    }
}

impl HttpSend for MockSender {
    async fn send(
        &self,
        _: reqwest_middleware::RequestBuilder,
    ) -> Result<reqwest::Response, reqwest_middleware::Error> {
        let response = response::Builder::new()
            .status(self.status)
            .body(self.body)
            .expect("Could not construct mocked response");
        Ok(response.into())
    }
}
