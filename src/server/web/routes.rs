use futures::future::{BoxFuture, FutureExt};
use log::{debug, error, info, warn};
use reqwest::StatusCode;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use warp::{Filter, Rejection, Reply};

use crate::server::db::db_manager::DBManager;
use crate::server::web::spt_api_proxy::ApiProxy;
use crate::util::errors::return_response_code;
use crate::util::errors::{ApiError, DBError};

#[derive(Debug, Clone, Eq, PartialEq)]
enum RouteType {
    Get,
    Post,
    Put,
    Delete,
}

type RouteFunc = for<'a> fn(
    &'a Arc<ApiProxy>,
    &'a Arc<DBManager>,
    Option<&'a HashMap<String, String>>, // query parameters
    Option<&'a Value>,                   // body
) -> BoxFuture<'a, Result<(StatusCode, Value), ApiError>>;

fn route_func_next<'a>(
    proxy: &'a Arc<ApiProxy>,
    db_manager: &'a Arc<DBManager>,
    params: Option<&'a HashMap<String, String>>, // query parameters
    body: Option<&'a Value>,                     // body
) -> BoxFuture<'a, Result<(StatusCode, Value), ApiError>> {
    async move {
        let res1 = proxy
            .get("me/player/currently-playing", params.cloned())
            .await;

        let res2 = proxy
            .post("me/player/next", body.cloned(), params.cloned())
            .await;

        if res1.as_ref().map_or(false, |r| r.0 == StatusCode::OK)
            && res2.as_ref().map_or(false, |r| r.0 == StatusCode::OK)
        {
            if let Some(user_id) = params.and_then(|p| p.get("user_id")).cloned() {
                if let Some(uri) = res1.unwrap().1["item"]["uri"].as_str() {
                    db_manager.increment_skip_count(user_id, uri.to_string());
                } else {
                    return Err(ApiError::ResponseDataError);
                }
            } else {
                return Err(ApiError::RequestError);
            }
        }

        return res2;
    }
    .boxed()
}

fn route_func_prev<'a>(
    proxy: &'a Arc<ApiProxy>,
    db_manager: &'a Arc<DBManager>,
    params: Option<&'a HashMap<String, String>>, // query parameters
    body: Option<&'a Value>,                     // body
) -> BoxFuture<'a, Result<(StatusCode, Value), ApiError>> {
    async move {
        let res2 = proxy
            .post("me/player/previous", body.cloned(), params.cloned())
            .await;

        let res1 = proxy
            .get("me/player/currently-playing", params.cloned())
            .await;

        if res1.as_ref().map_or(false, |r| r.0 == StatusCode::OK)
            && res2.as_ref().map_or(false, |r| r.0 == StatusCode::OK)
        {
            if let Some(user_id) = params.and_then(|p| p.get("user_id")).cloned() {
                if let Some(uri) = res1.unwrap().1["item"]["uri"].as_str() {
                    db_manager.increment_replay_count(user_id, uri.to_string());
                } else {
                    return Err(ApiError::ResponseDataError);
                }
            } else {
                return Err(ApiError::RequestError);
            }
        }

        return res2;
    }
    .boxed()
}

fn route_func_now_playing<'a>(
    proxy: &'a Arc<ApiProxy>,
    db_manager: &'a Arc<DBManager>,
    params: Option<&'a HashMap<String, String>>, // query parameters
    body: Option<&'a Value>,                     // body
) -> BoxFuture<'a, Result<(StatusCode, Value), ApiError>> {
    async move {
        proxy
            .get("me/player/currently-playing", params.cloned())
            .await
    }
    .boxed()
}

fn construct_json_fwd_route_no_body(
    route_type: RouteType,
    full_route: &str,
    api_proxies: Arc<RwLock<HashMap<u64, Arc<ApiProxy>>>>,
    db_manager: Arc<DBManager>,
    last_request_time: Arc<Mutex<Instant>>,
    route_function: Option<RouteFunc>,
) -> impl Filter<Extract = (warp::reply::WithStatus<warp::reply::Json>,), Error = Rejection> + Clone
{
    let full_route = full_route.to_string();
    let path_parts: Vec<_> = full_route.split('/').map(String::from).collect();
    let mut route = warp::any().boxed();
    for part in path_parts.iter() {
        route = route.and(warp::path(part.clone())).boxed();
    }

    if route_type == RouteType::Post
        || route_type == RouteType::Put
        || route_type == RouteType::Delete
    {
        error!("Cannot construct POST/PUT/DELETE route without body.");
        panic!();
    } else if route_type == RouteType::Get {
        route = route.and(warp::get()).boxed();
    }

    route
        .and(warp::path::end())
        .and(warp::query::<HashMap<String, String>>())
        .and_then({
            // let full_route = full_route.clone();
            // let route_type = route_type.clone();

            move |query: HashMap<String, String>| {
                let last_request_time = Arc::clone(&last_request_time);
                let api_proxies = Arc::clone(&api_proxies);
                let db_manager = Arc::clone(&db_manager);
                let route_type = route_type.clone();
                let full_route = full_route.clone();

                async move {
                    update_last_request_time(&last_request_time).await;

                    let client_id = query.get("client_id").and_then(|s| s.parse::<u64>().ok());
                    if client_id.is_none() {
                        error!("Received call to route /{} without client_id.", full_route);
                        return Ok::<_, warp::Rejection>(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({})),
                            warp::http::StatusCode::FORBIDDEN,
                        ));
                    }

                    info!(
                        "Received call to route /{} from client_id {}.",
                        full_route,
                        client_id.clone().unwrap()
                    );

                    let proxy = if let Some(id) = client_id {
                        let proxies = api_proxies.read().await;
                        proxies.get(&id).map(|p| Arc::clone(p))
                    } else {
                        None
                    };
                    if proxy.is_none() {
                        error!(
                            "No proxy found for client_id {}.",
                            client_id.clone().unwrap()
                        );
                        return Ok::<_, warp::Rejection>(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({})),
                            warp::http::StatusCode::FORBIDDEN,
                        ));
                    }

                    let proxy = proxy.unwrap();
                    let shortened_route = &full_route["api/spt-fwd/".len()..];
                    let res = match route_function.is_none() {
                        true => match route_type.clone() {
                            RouteType::Get => proxy.get(shortened_route, Some(query)).await,
                            RouteType::Delete => {
                                error!("Cannot construct DELETE route without body.");
                                proxy.delete(shortened_route, None, None).await
                            }
                            RouteType::Post => {
                                error!("Cannot construct POST route without body.");
                                proxy.post(shortened_route, None, None).await
                            }
                            RouteType::Put => {
                                error!("Cannot construct PUT route without body.");
                                proxy.put(shortened_route, None, None).await
                            }
                        },
                        false => {
                            route_function.unwrap()(&proxy, &db_manager, Some(&query), None).await
                        }
                    };

                    match res {
                        Ok((status, json)) => {
                            info!(
                                "Forwarding request to route /{} with status {}.",
                                full_route, status
                            );
                            Ok::<_, warp::Rejection>(warp::reply::with_status(
                                warp::reply::json(&json),
                                status,
                            ))
                        }
                        Err(err) => {
                            warn!(
                                "Forwarding request to route /{} with status {}.",
                                full_route,
                                return_response_code(err.clone())
                            );
                            Ok::<_, warp::Rejection>(warp::reply::with_status(
                                warp::reply::json(&serde_json::json!({
                                    "error": format!("Error: {}", err)
                                })),
                                return_response_code(err),
                            ))
                        }
                    }
                }
            }
        })
}

fn construct_json_fwd_route_with_body(
    route_type: RouteType,
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

    if route_type == RouteType::Get {
        // error
        error!("Cannot construct GET route with body.");
        panic!();
    } else if route_type == RouteType::Post {
        route = route.and(warp::post()).boxed();
    } else if route_type == RouteType::Put {
        route = route.and(warp::put()).boxed();
    } else if route_type == RouteType::Delete {
        route = route.and(warp::delete()).boxed();
    }

    route
        .and(warp::path::end())
        .and(warp::query::<HashMap<String, String>>())
        .and(warp::body::json::<serde_json::Value>())
        .and_then({
            let full_route = full_route.clone();
            let route_type = route_type.clone();

            move |query: HashMap<String, String>, body: serde_json::Value| {
                let last_request_time = Arc::clone(&last_request_time);
                let api_proxies = Arc::clone(&api_proxies);
                let full_route = full_route.clone();
                let route_type = route_type.clone();

                async move {
                    update_last_request_time(&last_request_time).await;

                    let client_id = body["client_id"].as_u64();
                    if client_id.is_none() {
                        error!("Received call to route /{} without client_id.", full_route);
                        return Ok::<_, warp::Rejection>(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({})),
                            warp::http::StatusCode::FORBIDDEN,
                        ));
                    }

                    info!(
                        "Received call to route /{} from client_id {}.",
                        full_route,
                        client_id.unwrap()
                    );

                    let proxy = if let Some(id) = client_id {
                        let proxies = api_proxies.read().await;
                        proxies.get(&id).map(Arc::clone)
                    } else {
                        None
                    };
                    if proxy.is_none() {
                        error!("No proxy found for client_id {}.", client_id.unwrap());
                        return Ok::<_, warp::Rejection>(warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({})),
                            warp::http::StatusCode::FORBIDDEN,
                        ));
                    }

                    let proxy = proxy.unwrap();
                    let shortened_route = &full_route["api/spt-fwd/".len()..];
                    let res = match route_type.clone() {
                        RouteType::Get => {
                            error!("Cannot construct GET route with body.");
                            proxy.get(shortened_route, None).await
                        }
                        RouteType::Put => proxy.put(shortened_route, Some(body), Some(query)).await,
                        RouteType::Post => {
                            proxy.post(shortened_route, Some(body), Some(query)).await
                        }
                        RouteType::Delete => {
                            proxy.delete(shortened_route, Some(body), Some(query)).await
                        }
                    };

                    match res {
                        Ok((status, json)) => {
                            info!(
                                "Forwarding request to route /{} with status {}.",
                                full_route, status
                            );
                            Ok::<_, warp::Rejection>(warp::reply::with_status(
                                warp::reply::json(&json),
                                status,
                            ))
                        }
                        Err(err) => {
                            warn!(
                                "Forwarding request to route /{} with status {}.",
                                full_route,
                                return_response_code(err.clone())
                            );
                            Ok::<_, warp::Rejection>(warp::reply::with_status(
                                warp::reply::json(&serde_json::json!({
                                    "error": format!("Error: {}", err)
                                })),
                                return_response_code(err),
                            ))
                        }
                    }
                }
            }
        })
}

fn construct_json_fwd_route(
    route_type: RouteType,
    full_route: &str,
    db_manager: Arc<DBManager>,
    api_proxies: Arc<RwLock<HashMap<u64, Arc<ApiProxy>>>>,
    last_request_time: Arc<Mutex<Instant>>,
    route_function: Option<RouteFunc>,
) -> impl Filter<Extract = (warp::reply::WithStatus<warp::reply::Json>,), Error = Rejection> + Clone
{
    if route_type == RouteType::Get {
        construct_json_fwd_route_no_body(
            RouteType::Get,
            full_route,
            api_proxies,
            db_manager,
            last_request_time,
            route_function,
        )
        .boxed()
    } else {
        construct_json_fwd_route_with_body(route_type, full_route, api_proxies, last_request_time)
            .boxed()
    }
}

pub fn routes(
    api_proxies: Arc<RwLock<HashMap<u64, Arc<ApiProxy>>>>,
    db_manager: Arc<DBManager>,
    next_client_id: Arc<Mutex<u64>>,
    last_request_time: Arc<Mutex<Instant>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    // Define the routes
    let json_fwd_get_routes = vec![
        "api/spt-fwd/me/player/currently-playing", // no db stuff
        "api/spt-fwd/me/player/devices",           // no db stuff
        "api/spt-fwd/me/player/queue",             // no db stuff
        "api/spt-fwd/me/player/recently-played",   // no db stuff
    ];

    let initial_route = construct_json_fwd_route(
        RouteType::Get,
        json_fwd_get_routes[0],
        Arc::clone(&db_manager),
        Arc::clone(&api_proxies),
        Arc::clone(&last_request_time),
        route_function_mapper(json_fwd_get_routes[0]),
    )
    .boxed();
    let api_routes = json_fwd_get_routes
        .iter()
        .skip(1)
        .map(|route| {
            construct_json_fwd_route(
                RouteType::Get,
                route,
                Arc::clone(&db_manager),
                Arc::clone(&api_proxies),
                Arc::clone(&last_request_time),
                route_function_mapper(route),
            )
        })
        .fold(initial_route, |acc, route| acc.or(route).unify().boxed());

    let json_fwd_put_routes = vec![
        "api/spt-fwd/me/player/play",  // no db stuff
        "api/spt-fwd/me/player/pause", // no db stuff
        "api/spt-fwd/me/player",       // no db stuff
    ];
    let api_routes = json_fwd_put_routes
        .iter()
        .map(|route| {
            construct_json_fwd_route(
                RouteType::Put,
                route,
                Arc::clone(&db_manager),
                Arc::clone(&api_proxies),
                Arc::clone(&last_request_time),
                route_function_mapper(json_fwd_get_routes[0]),
            )
        })
        .fold(api_routes, |acc, route| acc.or(route).unify().boxed());

    let json_fwd_post_routes = vec![
        "api/spt-fwd/me/player/next",     // db mark song skipped (analytics)
        "api/spt-fwd/me/player/previous", // db mark songs replayed - need to get recent (analytics)
        "api/spt-fwd/me/player/seek",     // no db stuff
        "api/spt-fwd/me/player/volume",   // no db stuff
        "api/spt-fwd/me/player/queue",    // no db stuff
    ];
    let api_routes = json_fwd_post_routes
        .iter()
        .map(|route| {
            construct_json_fwd_route(
                RouteType::Post,
                route,
                Arc::clone(&db_manager),
                Arc::clone(&api_proxies),
                Arc::clone(&last_request_time),
                route_function_mapper(json_fwd_get_routes[0]),
            )
        })
        .fold(api_routes, |acc, route| acc.or(route).unify().boxed());

    let ping_route = warp::path("ping").and(warp::path::end()).and_then({
        let last_request_time = Arc::clone(&last_request_time);
        move || {
            let last_request_time = Arc::clone(&last_request_time);
            info!("Received call to route /ping.",);
            async move {
                update_last_request_time(&last_request_time).await;
                let v: Value = serde_json::json!({ "status": "ok" });
                return Ok::<_, warp::Rejection>(warp::reply::json(&v));
            }
        }
    });

    let root_route = warp::path::end().and_then({
        let last_request_time = Arc::clone(&last_request_time);
        move || {
            info!("Received call to route /.",);
            let last_request_time = Arc::clone(&last_request_time);
            async move {
                update_last_request_time(&last_request_time).await;
                return Ok::<_, warp::Rejection>(warp::reply::html("SPT Server is running!"));
            }
        }
    });

    let init_route = warp::path("init").and(warp::path::end()).and_then({
        let last_request_time = Arc::clone(&last_request_time);
        let api_proxies = Arc::clone(&api_proxies);
        let next_client_id = Arc::clone(&next_client_id);

        move || {
            info!("Received call to route /init.",);

            let last_request_time = Arc::clone(&last_request_time);
            let api_proxies = Arc::clone(&api_proxies);
            let next_client_id = Arc::clone(&next_client_id);

            async move {
                update_last_request_time(&last_request_time).await;

                let mut next_client_id_g = next_client_id.lock().await;

                let client_id_val = *next_client_id_g;
                *next_client_id_g += 1;

                {
                    let mut api_proxies = api_proxies.write().await;
                    api_proxies.insert(client_id_val, Arc::new(ApiProxy::new(client_id_val)));

                    debug!("Added client_id {} to API proxy map.", client_id_val);

                    // println!("Printing the HashMap:");
                    // for (key, value) in &*api_proxies {
                    //     println!("{}: {:p}", key, &value);
                    // }
                }

                let v: Value = serde_json::json!({ "client_id": client_id_val });
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
                let last_request_time = Arc::clone(&last_request_time);
                let api_proxies = Arc::clone(&api_proxies);

                async move {
                    update_last_request_time(&last_request_time).await;

                    let client_id = query.get("state").and_then(|s| s.parse::<u64>().ok());
                    if client_id.is_none() {
                        error!("Received call to route /auth/cb without client_id.");
                        return Ok::<_, warp::Rejection>(warp::reply::html(
                            "Sorry, something went wrong.",
                        ));
                    }

                    info!(
                        "Received call to route /auth/cb with client_id {}.",
                        client_id.clone().unwrap()
                    );

                    let proxy = {
                        let api_proxies = api_proxies.read().await;
                        if let Some(p) = api_proxies.get(&client_id.unwrap()) {
                            Arc::clone(p)
                        } else {
                            error!(
                                "No proxy found for client_id {}.",
                                client_id.clone().unwrap()
                            );
                            return Ok::<_, warp::Rejection>(warp::reply::html(
                                "Sorry, something went wrong.",
                            ));
                        }
                    };

                    if let Some(code) = query.get("code") {
                        proxy.set_cb_auth_code(code.to_owned()).await;
                    } else {
                        error!(
                            "No callback authorization code found for client_id {}.",
                            client_id.clone().unwrap()
                        );
                        return Ok::<_, warp::Rejection>(warp::reply::html(
                            "Sorry, something went wrong.",
                        ));
                    }

                    info!(
                        "Set callback authorization code for client_id {}.",
                        client_id.clone().unwrap()
                    );

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

fn route_function_mapper(full_route: &str) -> Option<RouteFunc> {
    match full_route {
        "api/spt-fwd/me/player/next" => Some(route_func_next),
        "api/spt-fwd/me/player/previous" => Some(route_func_prev),
        _ => None,
    }
}

async fn update_last_request_time(last_request_time: &Arc<Mutex<Instant>>) {
    let mut last_time = last_request_time.lock().await;
    *last_time = Instant::now();
}
