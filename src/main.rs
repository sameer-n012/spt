use dotenvy::dotenv;
use tokio;

mod server {
    pub mod web {
        pub mod routes;
        pub mod server;
        pub mod spt_api_manager;
    }
}

mod util {
    pub mod errors;
    pub mod uri_helper;
}

mod client {
    pub mod local_api_manager;
    pub mod cli {
        pub mod cli_app;
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

    let mut api_manager = client::local_api_manager::ApiManager::new();

    client::cli::cli_app::run_cli(&mut api_manager).await;
}
