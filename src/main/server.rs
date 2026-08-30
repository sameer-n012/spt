use chrono::Local;
use dotenvy::dotenv;
use fern::Dispatch;
use log::{error, info, LevelFilter};
use spt::server;
use std::env;
use tokio;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize logging
    let log_file_name = format!(
        "logs/spt_server_{}.log",
        Local::now().format("%Y%m%d-%H%M%S")
    );
    let logger = Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}: {}",
                Local::now().format("%Y-%m-%d %H:%M:%S%.6f"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(LevelFilter::Warn)
        .level_for("spt", LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file(log_file_name).unwrap())
        .apply();

    if logger.is_err() {
        eprintln!("Failed to initialize logger: {:?}", logger.err());
    }

    info!("Starting server.");

    // Read port and timeout from environment or use defaults
    let port: u16 = env::var("SPT_SERVER_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    let inactivity_timeout_secs: u64 = env::var("SPT_SERVER_TIMEOUT_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);

    if let Err(e) = server::web::server::start_server(
        port,
        std::time::Duration::from_secs(inactivity_timeout_secs),
    )
    .await
    {
        error!("Server error: {}", e);
    }

    info!("Stopping server.");
}
