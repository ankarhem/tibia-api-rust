use once_cell::sync::Lazy;
use tibia_api::{app, run, telemetry, AppState};

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

pub fn spawn_app(state: AppState) -> std::net::SocketAddr {
    Lazy::force(&TRACING);

    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("To bind to random port");
    let addr = listener.local_addr().expect("To get local address");

    let app = app(state);
    tokio::spawn(run(app, listener));

    addr
}
