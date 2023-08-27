use anyhow::Result;
use std::net::TcpListener;
use tibia_api::telemetry;
use tracing_appender::rolling;
use tracing_subscriber::fmt::writer::MakeWriterExt;

#[tokio::main]
async fn main() -> Result<()> {
    let log_file = rolling::daily("./logs", "tibia_api.log");
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(log_file);
    let sink = std::io::stdout.and(non_blocking_writer);
    let subscriber = telemetry::get_subscriber("tibia_api".into(), "info".into(), sink);
    telemetry::init_subscriber(subscriber);

    let port = std::env::var("PORT").unwrap_or("3000".to_string());

    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))?;
    tibia_api::run(listener).await?;

    Ok(())
}
