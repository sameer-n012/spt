use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use warp::{Filter, Rejection, Reply};

use crate::server::web::spt_api_proxy::ApiProxy;
use crate::util::errors::return_response_code;

// fn construct_json_fwd_route(
//     route: &str,
//     server_meta: Arc<Mutex<ServerMeta>>,
//     last_request_time: Arc<Mutex<Instant>>,
// ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
//     let rt = route
//         .clone()
//         .split("/")
//         .map(|x| warp::path(x))
//         .fold(warp::path("api").and("spt-fwd"), |acc, x| acc.and(x))
//         .and(warp::path::end());

//     let route = rt.and_then({
//         let last_request_time = Arc::clone(&last_request_time);
//         let api_manager = {
//             let m = server_meta.lock().unwrap();
//             m.api_manager.clone()
//         };

//         move || {
//             let last_request_time = Arc::clone(&last_request_time);
//             let mut api_manager = api_manager.clone();

//             async move {
//                 update_last_request_time(&last_request_time);
//                 let res = api_manager.get(route, None).await;

//                 match res {
//                     Ok((status, json)) => Ok::<_, warp::Rejection>(warp::reply::json(&json)),
//                     Err(err) => Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
//                         "error": format!("Error: {}", err)
//                     }))),
//                 }
//             }
//         }
//     });

//     return route;
// }

pub fn routes(
    api_proxies: Arc<HashMap<u64, Arc<ApiProxy>>>,
    next_client_id: Arc<Mutex<u64>>,
    last_request_time: Arc<Mutex<Instant>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    // Define the routes
    //
    // let get_routes = vec!["me/player/currently-playing"];
    // let get_routes = get_routes
    //     .iter()
    //     .map(|x| {
    //         construct_json_fwd_route(x, Arc::clone(&server_meta), Arc::clone(&last_request_time))
    //     })
    //     .reduce(|acc, x| acc.or(x));
    //
    // let now_route = construct_json_fwd_route(
    //     "me/player/currently-playing",
    //     Arc::clone(&server_meta),
    //     Arc::clone(&last_request_time),
    // );

    let now_route = warp::path("api")
        .and(warp::path("spt-fwd"))
        .and(warp::path("me"))
        .and(warp::path("player"))
        .and(warp::path("currently-playing"))
        .and(warp::path::end())
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and_then({
            println!("here -1");

            let last_request_time = Arc::clone(&last_request_time);
            let api_proxies = Arc::clone(&api_proxies);

            move |query: std::collections::HashMap<String, String>| {
                let last_request_time = Arc::clone(&last_request_time);
                let api_proxies = Arc::clone(&api_proxies);

                async move {
                    update_last_request_time(&last_request_time).await;

                    println!("api_manager pointer in now route {:p}", &api_proxies);

                    let proxy = query
                        .get("client_id")
                        .and_then(|s| s.parse::<u64>().ok())
                        .and_then(|id| api_proxies.get(&id));
                    if proxy.is_none() {
                        return Ok::<_, warp::Rejection>(warp::reply::with_status(
                            warp::reply(),
                            warp::http::StatusCode::FORBIDDEN,
                        ));
                    }

                    let proxy = proxy.unwrap();

                    let res = proxy.get("me/player/currently-playing", None).await;

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
        });

    let ping_route = warp::path("ping").and(warp::path::end()).and_then({
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

    let init_route = warp::path("init").and(warp::path::end()).and_then({
        let last_request_time = Arc::clone(&last_request_time);
        let api_managers = Arc::clone(&api_proxies);
        move || {
            let last_request_time = Arc::clone(&last_request_time);
            let api_managers = Arc::clone(&api_managers);

            async move {
                update_last_request_time(&last_request_time).await;

                let next_client_id_g = next_client_id.lock().await;

                let client_id_val = *next_client_id_g;
                *next_client_id_g += 1;

                api_managers.insert(client_id_val, Arc::new(ApiProxy::new(client_id_val)));

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
                    // temporarily get client_id=1:
                    let client_id = 1;
                    if let Some(proxy) = api_proxies.get(&client_id) {
                        if let Some(code) = query.get("code") {
                            println!("Received auth code: {:?}", code);
                            proxy.set_cb_auth_code(code.to_owned()).await;
                        }
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

    return now_route.or(ping_route).or(init_route).or(auth_cb_route);
}

async fn update_last_request_time(last_request_time: &Arc<Mutex<Instant>>) {
    let mut last_time = last_request_time.lock().await;
    *last_time = Instant::now();
}
