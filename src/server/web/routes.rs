use reqwest::StatusCode;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, RwLock};
use warp::{any, Filter, Rejection, Reply};

use crate::server::web::spt_api_proxy::ApiProxy;
use crate::util::errors::return_response_code;

fn construct_json_fwd_get_route(
    full_route: &str,
    api_proxies: Arc<RwLock<HashMap<u64, Arc<ApiProxy>>>>,
    last_request_time: Arc<Mutex<Instant>>,
) -> impl Filter<Extract = (warp::reply::WithStatus<warp::reply::Json>,), Error = Rejection> + Clone
{
    let full_route = full_route.to_string();
    let path_parts: Vec<_> = full_route.split('/').map(String::from).collect();
    let mut route = warp::any().boxed();
    for part in path_parts.iter() {
        route = route.and(warp::path(part.clone())).boxed();
    }

    // test
    // let a = any().and(warp::path("hi")).and(warp::path("there"));

    route
        .and(warp::path::end())
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and_then({
            let full_route = full_route.clone();

            move |query: std::collections::HashMap<String, String>| {
                let last_request_time = Arc::clone(&last_request_time);
                let api_proxies = Arc::clone(&api_proxies);

                let full_route = full_route.clone();

                async move {
                    update_last_request_time(&last_request_time).await;

                    let client_id = query.get("client_id").and_then(|s| s.parse::<u64>().ok());
                    if client_id.is_none() {
                        return Ok::<_, warp::Rejection>(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({})),
                            warp::http::StatusCode::FORBIDDEN,
                        ));
                    }

                    let proxy = if let Some(id) = client_id {
                        let proxies = api_proxies.read().await;
                        proxies.get(&id).map(|p| Arc::clone(p))
                    } else {
                        None
                    };
                    if proxy.is_none() {
                        return Ok::<_, warp::Rejection>(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({})),
                            warp::http::StatusCode::FORBIDDEN,
                        ));
                    }

                    let proxy = proxy.unwrap();
                    let shortened_route = &full_route["api/spt-fwd/".len()..];
                    let res = proxy.get(shortened_route, None).await;

                    match res {
                        Ok((status, json)) => Ok::<_, warp::Rejection>(warp::reply::with_status(
                            warp::reply::json(&json),
                            status,
                        )),
                        Err(err) => Ok::<_, warp::Rejection>(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({
                                "error": format!("Error: {}", err)
                            })),
                            return_response_code(err),
                        )),
                    }
                }
            }
        })
}

pub fn routes(
    api_proxies: Arc<RwLock<HashMap<u64, Arc<ApiProxy>>>>,
    next_client_id: Arc<Mutex<u64>>,
    last_request_time: Arc<Mutex<Instant>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    // Define the routes

    let json_fwd_get_routes = vec!["api/spt-fwd/me/player/currently-playing"];

    let initial_route = construct_json_fwd_get_route(
        json_fwd_get_routes[0],
        Arc::clone(&api_proxies),
        Arc::clone(&last_request_time),
    )
    .boxed();
    let api_routes = json_fwd_get_routes
        .iter()
        .skip(1)
        .map(|route| {
            construct_json_fwd_get_route(
                route,
                Arc::clone(&api_proxies),
                Arc::clone(&last_request_time),
            )
        })
        .fold(initial_route, |acc, route| acc.or(route).unify().boxed());

    // let now_route = warp::path("api")
    //     .and(warp::path("spt-fwd"))
    //     .and(warp::path("me"))
    //     .and(warp::path("player"))
    //     .and(warp::path("currently-playing"))
    //     .and(warp::path::end())
    //     .and(warp::query::<std::collections::HashMap<String, String>>())
    //     .and_then({
    //         println!("FFFFF: api/spt-fwd/me/player/currently-playing route");
    //         println!("here -1");

    //         let last_request_time = Arc::clone(&last_request_time);
    //         let api_proxies = Arc::clone(&api_proxies);

    //         move |query: std::collections::HashMap<String, String>| {
    //             let last_request_time = Arc::clone(&last_request_time);
    //             let api_proxies = Arc::clone(&api_proxies);

    //             async move {
    //                 update_last_request_time(&last_request_time).await;

    //                 println!("api_manager pointer in now route {:p}", &api_proxies);

    //                 let client_id = query.get("client_id").and_then(|s| s.parse::<u64>().ok());
    //                 if client_id.is_none() {
    //                     return Ok::<_, warp::Rejection>(warp::reply::with_status(
    //                         warp::reply::json(&serde_json::json!({})), // TODO figure out how to make just warp::reply()
    //                         warp::http::StatusCode::FORBIDDEN,
    //                     ));
    //                 }

    //                 let proxy = if let Some(id) = client_id {
    //                     let proxies = api_proxies.read().await;
    //                     proxies.get(&id).map(|p| Arc::clone(p))
    //                 } else {
    //                     None
    //                 };
    //                 if proxy.is_none() {
    //                     return Ok::<_, warp::Rejection>(warp::reply::with_status(
    //                         warp::reply::json(&serde_json::json!({})), // TODO figure out how to make just warp::reply()
    //                         warp::http::StatusCode::FORBIDDEN,
    //                     ));
    //                 }

    //                 let proxy = proxy.unwrap();

    //                 let res = proxy.get("me/player/currently-playing", None).await;

    //                 match res {
    //                     Ok((status, json)) => Ok::<_, warp::Rejection>(warp::reply::with_status(
    //                         warp::reply::json(&json),
    //                         status,
    //                     )),
    //                     Err(err) => Ok::<_, warp::Rejection>(warp::reply::with_status(
    //                         warp::reply::json(&serde_json::json!({
    //                             "error": format!("Error: {}", err)
    //                         })),
    //                         return_response_code(err),
    //                     )),
    //                 }
    //             }
    //         }
    //     });

    let ping_route = warp::path("ping").and(warp::path::end()).and_then({
        println!("FFFFF: /ping route");
        let last_request_time = Arc::clone(&last_request_time);
        move || {
            let last_request_time = Arc::clone(&last_request_time);
            async move {
                update_last_request_time(&last_request_time).await;
                let v: Value = serde_json::json!({ "status": "ok" });
                return Ok::<_, warp::Rejection>(warp::reply::json(&v));
            }
        }
    });

    let root_route = warp::path::end().and_then({
        println!("FFFFF: / route");
        let last_request_time = Arc::clone(&last_request_time);
        move || {
            let last_request_time = Arc::clone(&last_request_time);
            async move {
                update_last_request_time(&last_request_time).await;
                return Ok::<_, warp::Rejection>(warp::reply::html("SPT Server is running!"));
            }
        }
    });

    let init_route = warp::path("init").and(warp::path::end()).and_then({
        println!("FFFFF: /init route");
        let last_request_time = Arc::clone(&last_request_time);
        let api_proxies = Arc::clone(&api_proxies);
        let next_client_id = Arc::clone(&next_client_id);

        move || {
            println!("FFFFF: /init route 2");
            let last_request_time = Arc::clone(&last_request_time);
            let api_proxies = Arc::clone(&api_proxies);
            let next_client_id = Arc::clone(&next_client_id);

            async move {
                println!("FFFFF: /init route 3");
                update_last_request_time(&last_request_time).await;

                let mut next_client_id_g = next_client_id.lock().await;

                let client_id_val = *next_client_id_g;
                *next_client_id_g += 1;

                {
                    let mut api_proxies = api_proxies.write().await;
                    api_proxies.insert(client_id_val, Arc::new(ApiProxy::new(client_id_val)));
                    println!("FFFFF: /init route 4");

                    println!("Printing the HashMap:");
                    for (key, value) in &*api_proxies {
                        println!("{}: {:p}", key, &value);
                    }
                }

                let v: Value = serde_json::json!({"client_id": client_id_val});
                return Ok::<_, warp::Rejection>(warp::reply::json(&v));
            }
        }
    });

    let auth_cb_route = warp::path("auth")
        .and(warp::path("cb"))
        .and(warp::path::end())
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and_then({
            println!("FFFFF: /auth/cb route");
            let api_proxies = Arc::clone(&api_proxies);
            let last_request_time = Arc::clone(&last_request_time);

            move |query: std::collections::HashMap<String, String>| {
                println!("Query EEEE: {:?}", query);
                let last_request_time = Arc::clone(&last_request_time);
                let api_proxies = Arc::clone(&api_proxies);

                async move {
                    update_last_request_time(&last_request_time).await;

                    // let code = query.get("code").map(|s| s.to_owned());
                    // let api_manager = &mut server_meta.lock().unwrap().api_manager; // Borrow the ApiManager
                    // println!("Received auth code: {:?}", code);

                    // TODO get sent state which contains client_id and access
                    // the corresponding ApiProxy in the hash map
                    // then set the ApiProxy's cb_auth_code and notify
                    let client_id = query.get("state").and_then(|s| s.parse::<u64>().ok());
                    if client_id.is_none() {
                        return Ok::<_, warp::Rejection>(warp::reply::html(
                            "Sorry, something went wrong.",
                        ));
                    }

                    let proxy = {
                        let api_proxies = api_proxies.read().await;
                        if let Some(p) = api_proxies.get(&client_id.unwrap()) {
                            Arc::clone(p)
                        } else {
                            return Ok::<_, warp::Rejection>(warp::reply::html(
                                "Sorry, something went wrong.",
                            ));
                        }
                    };

                    if let Some(code) = query.get("code") {
                        println!("Received auth code: {:?}", code);
                        proxy.set_cb_auth_code(code.to_owned()).await;
                    } else {
                        return Ok::<_, warp::Rejection>(warp::reply::html(
                            "Sorry, something went wrong.",
                        ));
                    }

                    // OLD
                    // Set the cb_auth_code and notify the waiting task
                    // api_manager.cb_auth_code = code;
                    // println!("Set auth code: {:?}", api_manager.cb_auth_code);
                    // api_manager.cb_auth_notifier.notify_one(); // Notify that the code is ready

                    // if let Some(code) = query.get("code") {
                    //     {
                    //         let api_manager =
                    //             &mut server_meta.lock().unwrap().api_manager.write().unwrap();
                    //         api_manager.cb_auth_code = Some(code.clone());
                    //         println!("ApiManager pointer in callback: {:p}", api_manager);
                    //         println!(
                    //             "Received auth code and set it: {:?}",
                    //             api_manager.cb_auth_code
                    //         );
                    //     }

                    //     {
                    //         let api_manager =
                    //             &mut server_meta.lock().unwrap().api_manager.read().unwrap();
                    //         api_manager.cb_auth_notifier.notify_one();
                    //     }
                    // }

                    return Ok::<_, warp::Rejection>(warp::reply::html(
                        "Authorization received. You may close this tab.",
                    ));
                }
            }
        });

    return api_routes
        // .or(now_route)
        .or(ping_route)
        .or(init_route)
        .or(auth_cb_route)
        .or(root_route);
}

async fn update_last_request_time(last_request_time: &Arc<Mutex<Instant>>) {
    let mut last_time = last_request_time.lock().await;
    *last_time = Instant::now();
}
