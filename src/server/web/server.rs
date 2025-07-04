use std::env;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::oneshot;
use tokio::time;

use crate::server::web::routes;
use crate::server::web::spt_api_manager::ApiManager;

#[derive(Debug)]
pub struct ServerMeta {
    pub port: u16,
    pub inactivity_timeout: Duration,
    pub db_url: String,
    pub db_port: u16,
    pub api_manager: Arc<RwLock<ApiManager>>,
}

pub async fn start_server(
    port: u16,
    inactivity_timeout: Duration,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("URL: {}", "AAAA");

    // Shared state to track the last request time
    let last_request_time = Arc::new(Mutex::new(Instant::now()));
    // let last_request_time_clone = Arc::clone(&last_request_time);

    let server_meta = Arc::new(Mutex::new(ServerMeta {
        port,
        inactivity_timeout,
        db_url: env::var("DB_URL").expect("DB_URL must be set"),
        db_port: env::var("DB_PORT")
            .expect("DB_PORT must be set")
            .parse::<u16>()
            .unwrap(),
        api_manager: Arc::new(RwLock::new(ApiManager::new())),
    }));

    // Shutdown signal
    let (_, shutdown_rx) = oneshot::channel();

    let routes = routes::routes(server_meta, Arc::clone(&last_request_time));

    // Start the server with graceful shutdown
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let (addr, server) = warp::serve(routes).bind_with_graceful_shutdown(
        addr,
        handle_shutdown(
            shutdown_rx,
            Arc::clone(&last_request_time),
            inactivity_timeout,
        ),
    );

    println!("Server running at http://{}/", addr);
    env::set_var("SPT_RUST_APP_SERVER_RUNNING", "1");
    tokio::spawn(server);

    Ok(())
}

async fn check_inactive(last_request_time: Arc<Mutex<Instant>>, timeout: Duration) {
    loop {
        time::sleep(timeout).await;
        let last_time = *last_request_time.lock().unwrap();
        if last_time.elapsed() >= timeout {
            break;
        }
    }
}

async fn handle_shutdown(
    shutdown_rx: oneshot::Receiver<()>,
    last_request_time: Arc<Mutex<Instant>>,
    inactivity_timeout: Duration,
) {
    tokio::select! {
        // _ = shutdown_rx => {
        //     println!("Shutdown signal received, closing server.");
        // }
        _ = check_inactive(last_request_time, inactivity_timeout) => {
            println!("Server shut down due to inactivity.");
        }
    }
}

// #[tokio::main]
// async fn main() {
//     let port = 3030;
//     let inactivity_timeout = Duration::from_secs(60); // 1-minute inactivity timeout

//     if let Err(e) = start_server(port, inactivity_timeout).await {
//         eprintln!("Server error: {}", e);
//     }
// }
