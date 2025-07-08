use log::info;
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};

// use tokio::sync::oneshot;
use tokio::time;

use crate::server::web::routes;
use crate::server::web::spt_api_proxy::ApiProxy;

#[derive(Debug)]
pub struct ServerMeta {
    pub port: u16,
    pub inactivity_timeout: Duration,
    // pub db_url: String,
    // pub db_port: u16,
    pub api_proxies: Arc<RwLock<HashMap<u64, Arc<ApiProxy>>>>,
    pub next_client_id: Arc<Mutex<u64>>,
    pub last_request_time: Arc<Mutex<Instant>>,
}

pub async fn start_server(
    port: u16,
    inactivity_timeout: Duration,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Shared state to track the last request time
    let last_request_time = Arc::new(Mutex::new(Instant::now()));
    // let last_request_time_clone = Arc::clone(&last_request_time);

    let server_meta = ServerMeta {
        port,
        inactivity_timeout,
        // db_url: env::var("DB_URL").expect("DB_URL must be set"),
        // db_port: env::var("DB_PORT")
        //     .expect("DB_PORT must be set")
        //     .parse::<u16>()
        //     .unwrap(),
        api_proxies: Arc::new(RwLock::new(HashMap::new())),
        next_client_id: Arc::new(Mutex::new(1)),
        last_request_time: last_request_time,
    };

    // Shutdown signal - TODO delete
    // let (_, shutdown_rx) = oneshot::channel();

    let routes = routes::routes(
        Arc::clone(&server_meta.api_proxies),
        Arc::clone(&server_meta.next_client_id),
        Arc::clone(&server_meta.last_request_time),
    );

    // Start the server with graceful shutdown
    let addr = SocketAddr::from(([127, 0, 0, 1], server_meta.port));
    let (addr, server) = warp::serve(routes).bind_with_graceful_shutdown(
        addr,
        handle_shutdown(
            // shutdown_rx,
            Arc::clone(&server_meta.last_request_time),
            server_meta.inactivity_timeout.clone(),
        ),
    );

    tokio::spawn(server);

    env::set_var("SPT_RUST_APP_SERVER_RUNNING", "1"); // TODO delete (unsafe)

    info!("Server running at http://{}/.", addr);

    Ok(())
}

async fn check_inactive(last_request_time: Arc<Mutex<Instant>>, timeout: Duration) {
    loop {
        time::sleep(timeout).await;
        let last_time = *last_request_time.lock().await;
        if last_time.elapsed() >= timeout {
            break;
        }
    }
}

async fn handle_shutdown(
    // shutdown_rx: oneshot::Receiver<()>,
    last_request_time: Arc<Mutex<Instant>>,
    inactivity_timeout: Duration,
) {
    tokio::select! {
        // _ = shutdown_rx => {
        //     println!("Shutdown signal received, closing server.");
        // }
        _ = check_inactive(last_request_time, inactivity_timeout) => {
            info!("Server shut down due to inactivity after {}s.", inactivity_timeout.as_secs());
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
