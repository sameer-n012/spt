use chrono::Local;
use dotenvy::dotenv;
use fern::Dispatch;
use log::{error, info, LevelFilter};
use tokio;

mod server {
    pub mod web {
        pub mod routes;
        pub mod server;
        pub mod spt_api_proxy;
    }
}

mod util {
    pub mod errors;
    pub mod uri_helper;
}

mod client {
    pub mod local_api_proxy;
    pub mod cli {
        pub mod cli_app;
        pub mod eval;
        pub mod parser;
    }
    pub mod core {
        pub mod playback_manager;
        // pub mod playlist_manager;
        // pub mod queue_manager;
        // pub mod search_manager;
        // pub mod status_manager;
        // pub mod transaction_manager;
    }
}

#[tokio::main]
async fn main() {
    // load environment variables from .env file
    dotenv().ok();

    // initialize logging
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

    info!("Starting program.");

    let mut api_proxy = client::local_api_proxy::ApiProxy::new();
    if let Err(e) = api_proxy.setup().await {
        error!("Failed to set up API proxy: {}", e);
        return;
    }

    let args = std::env::args().collect::<Vec<String>>();
    client::cli::cli_app::run_cli(&mut api_proxy, args).await;

    info!("Stopping program.");
}
