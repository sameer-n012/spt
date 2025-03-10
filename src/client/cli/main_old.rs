use dotenvy::dotenv;
use std::env;
use tokio;
use web::api_manager::ApiManager;

mod core {
    pub mod playback_manager;
    // pub mod playlist_manager;
    // pub mod queue_manager;
    // pub mod search_manager;
    // pub mod status_manager;
    // pub mod transaction_manager;
}

mod web {
    pub mod api_manager;
    pub mod cb_server;
}

mod cli {
    pub mod cli_app;
}

mod util {
    pub mod uri_helper;
}

#[tokio::main]
async fn main() {
    // Initialize the API manager with the token
    // Example call to get devices
    // match api_manager.get_devices().await {
    //     Ok((status, json)) => {
    //         println!("Status: {:?}", status);
    //         println!("Response JSON: {:?}", json);
    //     }
    //     Err(e) => {
    //         eprintln!("Error occurred: {:?}", e);
    //     }
    // }
    //

    dotenv().ok();

    let mut api_manager = ApiManager::new();

    cli::cli_app::run_cli(&mut api_manager).await;
}
