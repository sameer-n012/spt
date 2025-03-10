use ::std::sync::{Arc, Mutex};
use serde_json::Value;
use std::time::Instant;
use warp::{reply::Response, Filter, Rejection, Reply};

use crate::server::web::server::ServerMeta;
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
    server_meta: Arc<Mutex<ServerMeta>>,
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
        .and_then({
            let last_request_time = Arc::clone(&last_request_time);
            let api_manager = {
                let m = server_meta.lock().unwrap();
                m.api_manager.clone()
            };

            move || {
                let last_request_time = Arc::clone(&last_request_time);
                let mut api_manager = api_manager.clone();

                async move {
                    update_last_request_time(&last_request_time);
                    let res = api_manager.get("me/player/currently-playing", None).await;

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

    let ping_route = warp::path("ping").and(warp::path::end()).map({
        let last_request_time = Arc::clone(&last_request_time);
        move || {
            update_last_request_time(&last_request_time);
            let v: Value = serde_json::json!({ "status": "ok" });
            return warp::reply::json(&v);
        }
    });

    let hello_route = warp::path("hello").and(warp::path::end()).map({
        let last_request_time = Arc::clone(&last_request_time);
        move || {
            update_last_request_time(&last_request_time);
            return warp::reply::html("Hello, world!");
        }
    });

    let goodbye_route = warp::path("goodbye").and(warp::path::end()).map({
        let last_request_time = Arc::clone(&last_request_time);
        move || {
            update_last_request_time(&last_request_time);
            return warp::reply::html("Goodbye!");
        }
    });

    let auth_cb_route = warp::path("auth")
        .and(warp::path("cb"))
        .and(warp::path::end())
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .map({
            let server_meta = Arc::clone(&server_meta);
            move |query: std::collections::HashMap<String, String>| {
                let code = query.get("code").map(|s| s.to_owned());
                let api_manager = &mut server_meta.lock().unwrap().api_manager; // Borrow the ApiManager

                // Set the cb_auth_code and notify the waiting task
                api_manager.cb_auth_code = code;
                api_manager.cb_auth_notifier.notify_one(); // Notify that the code is ready

                update_last_request_time(&last_request_time);
                return warp::reply::html("Authorization received. You may close this tab.");
            }
        });

    return now_route
        .or(ping_route)
        .or(hello_route)
        .or(goodbye_route)
        .or(auth_cb_route);
}

fn update_last_request_time(last_request_time: &Arc<Mutex<Instant>>) {
    let mut last_time = last_request_time.lock().unwrap();
    *last_time = Instant::now();
}
