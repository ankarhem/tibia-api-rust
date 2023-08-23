use anyhow::Result;
use std::net::TcpListener;
use tibia_api::telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = telemetry::get_subscriber("tibia_api".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let port = std::env::var("PORT").unwrap_or("3000".to_string());

    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))?;
    tibia_api::run(listener).await?;

    Ok(())
}
