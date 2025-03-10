use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::{oneshot, watch};
use tokio::time::{self, Duration};
use warp::Filter;

pub async fn start_callback_server(port: u16) -> Result<String, Box<dyn std::error::Error>> {
    // Channel for notifying when we've received the callback
    let (tx, rx) = oneshot::channel();
    let tx = Arc::new(Mutex::new(Some(tx)));

    // Watch channel for graceful shutdown
    let (shutdown_tx, shutdown_rx) = watch::channel(());

    // Route that handles the callback
    let route = warp::get()
        .and(warp::path::end())
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .map(move |query: std::collections::HashMap<String, String>| {
            // Extract the authorization code from the query
            if let Some(code) = query.get("code") {
                // Send the code back to the main task and close the server
                if let Some(tx) = tx.lock().unwrap().take() {
                    let _ = tx.send(code.clone());
                }
                format!("Authorization received. You may close this tab.")
            } else {
                "No authorization code found.".to_string()
            }
        });

    // Create a socket address for the server
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    // Duration for graceful shutdown
    let timeout_duration = Duration::from_secs(60);

    // Start the server with graceful shutdown
    let (addr, server) = warp::serve(route).bind_with_graceful_shutdown(addr, async move {
        // tokio::select! {
        //     _ = shutdown_rx => {
        //         println!("Received callback, shutting down server.");
        //     },
        //     _ = time::sleep(timeout_duration) => {
        //         println!("Timeout reached, shutting down server.");
        //     },
        // }
        //
        // Shutdown when a signal is sent on `shutdown_rx`
        shutdown_rx.clone().changed().await.ok();
    });

    println!("Callback server running at http://{}/", addr);
    tokio::spawn(server);

    // Wait until we receive the code
    match rx.await {
        Ok(code) => {
            // Signal shutdown to the server
            let _ = shutdown_tx.send(());
            Ok(code)
        }
        Err(_) => Err("Failed to receive authorization code.".into()),
    }
}
