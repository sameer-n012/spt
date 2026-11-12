use log::info;
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};

// use tokio::sync::oneshot;
use tokio::time;

use crate::server::db::db_manager::DBManager;
use crate::server::web::routes;
use crate::server::web::spt_api_proxy::ApiProxy;

#[derive(Debug)]
pub struct ServerMeta {
    pub port: u16,
    pub inactivity_timeout: Duration,
    pub db_manager: Arc<DBManager>,
    pub api_proxies: Arc<RwLock<HashMap<u64, Arc<ApiProxy>>>>,
    pub next_client_id: Arc<Mutex<u64>>,
    pub last_request_time: Arc<Mutex<Instant>>,
    pub client_last_activity: Arc<Mutex<HashMap<u64, Instant>>>,
}

pub async fn start_server(
    port: u16,
    inactivity_timeout: Duration,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Shared state to track the last request time
    let last_request_time = Arc::new(Mutex::new(Instant::now()));
    let client_last_activity = Arc::new(Mutex::new(HashMap::new()));

    let db_url = env::var("SPT_DB_URL").unwrap_or("".to_string());
    let db_port: u16 = env::var("SPT_DB_PORT")
        .unwrap_or("0".to_string())
        .parse()
        .unwrap_or(0);

    let mut db_manager = DBManager::new(db_url, db_port);
    db_manager.setup().await?;

    let server_meta = ServerMeta {
        port,
        inactivity_timeout,
        db_manager: Arc::new(db_manager),
        api_proxies: Arc::new(RwLock::new(HashMap::new())),
        next_client_id: Arc::new(Mutex::new(1)),
        last_request_time,
        client_last_activity: Arc::clone(&client_last_activity),
    };

    // Shutdown signal - TODO delete
    // let (_, shutdown_rx) = oneshot::channel();

    let routes = routes::routes(
        Arc::clone(&server_meta.api_proxies),
        Arc::clone(&server_meta.db_manager),
        Arc::clone(&server_meta.next_client_id),
        Arc::clone(&server_meta.last_request_time),
        Arc::clone(&server_meta.client_last_activity),
    );

    // Start the server with graceful shutdown
    let addr = SocketAddr::from(([127, 0, 0, 1], server_meta.port));
    let (addr, server) = warp::serve(routes).bind_with_graceful_shutdown(
        addr,
        handle_shutdown(
            // shutdown_rx,
            Arc::clone(&server_meta.last_request_time),
            server_meta.inactivity_timeout,
        ),
    );

    // tokio::spawn(server);
    tokio::spawn(check_periodic_update(
        Arc::clone(&server_meta.api_proxies),
        Arc::clone(&server_meta.db_manager),
        Arc::clone(&server_meta.client_last_activity),
        server_meta.inactivity_timeout,
        Duration::from_secs(10),
    ));

    env::set_var("SPT_RUST_APP_SERVER_RUNNING", "1"); // TODO delete (unsafe)

    info!("Server running at http://{}/.", addr);

    server.await;

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

// call now-playing status update function every 10s
async fn check_periodic_update(
    api_proxies: Arc<RwLock<HashMap<u64, Arc<ApiProxy>>>>,
    db_manager: Arc<DBManager>,
    client_last_activity: Arc<Mutex<HashMap<u64, Instant>>>,
    inactivity_timeout: Duration,
    update_interval: Duration,
) {
    let mut interval = time::interval(update_interval);
    loop {
        interval.tick().await;
        periodic_update(Arc::clone(&api_proxies), Arc::clone(&db_manager)).await;

        // Remove inactive clients
        let now = Instant::now();
        let mut last_activity = client_last_activity.lock().await;
        let mut proxies = api_proxies.write().await;
        let inactive_ids: Vec<u64> = last_activity
            .iter()
            .filter_map(|(id, last)| {
                if now.duration_since(*last) > inactivity_timeout {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();
        for id in inactive_ids {
            proxies.remove(&id);
            last_activity.remove(&id);
            info!("Removed inactive client_id {} due to inactivity.", id);
        }
    }
}

async fn periodic_update(
    // last_request_time: Arc<Mutex<Instant>>,
    api_proxies: Arc<RwLock<HashMap<u64, Arc<ApiProxy>>>>,
    db_manager: Arc<DBManager>,
) {
    // update last request time
    // let mut last_time = last_request_time.lock().await;
    // *last_time = Instant::now();

    let proxies = api_proxies.read().await;
    for (_id, proxy) in proxies.iter() {
        // info!("Checking now-playing status for client {}", _id);
        let res = proxy.get("me/player/currently-playing", None).await;
        if let Ok((status, json)) = res {
            if status != 200 {
                continue;
            }
            if let Some(item) = json.get("item") {
                if let Some(uri) = item.get("uri").and_then(|u| u.as_str()) {
                    if let Some(user_id) = item.get("user_id").and_then(|u| u.as_str()) {
                        let position = json
                            .get("progress_ms")
                            .and_then(|p| p.as_u64())
                            .unwrap_or(0) as i64;
                        let _ = db_manager.update_now_playing(user_id, uri, position).await;
                    }
                }
            }
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
