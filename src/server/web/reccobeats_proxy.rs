use crate::server::db::db_manager::DBManager;
use crate::util::errors::{self, ApiError};
use crate::util::uri_helper::get_id_from_uri;
use log::{debug, warn};
use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

// audio features from reccobeats
#[derive(Debug)]
pub struct ReccoBeatsProxy {
    client: Client,
    base_url: String,
    backoff: RwLock<SystemTime>, // time to start api calls again
}

impl ReccoBeatsProxy {
    pub fn new(base_url: String) -> Self {
        return ReccoBeatsProxy {
            client: Client::new(),
            base_url,
            backoff: RwLock::new(SystemTime::now()),
        };
    }

    // audio features for one spotify track id, none if reccobeats doesn't know the track
    pub async fn audio_features(&self, spotify_id: &str) -> Result<Option<Value>, ApiError> {
        // features are best effort, skip rather than wait out a backoff
        if *self.backoff.read().await > SystemTime::now() {
            return Err(ApiError::ResponseError429);
        }

        // /v1/track/{id}/audio-features only takes reccobeats ids, this route takes spotify ids
        let url = format!("{}/v1/audio-features", self.base_url);
        let response = match self
            .client
            .get(&url)
            .query(&[("ids", spotify_id)])
            .send()
            .await
        {
            Ok(res) => res,
            Err(_) => return Err(ApiError::RequestError),
        };

        let status = response.status();
        match status {
            StatusCode::OK => {
                let json: Value = response
                    .json()
                    .await
                    .map_err(|_| ApiError::ResponseParseError)?;
                // results don't echo the requested id, match on the spotify url instead
                let suffix = format!("/track/{}", spotify_id);
                return Ok(json["content"].as_array().and_then(|items| {
                    items
                        .iter()
                        .find(|item| item["href"].as_str().is_some_and(|h| h.ends_with(&suffix)))
                        .cloned()
                }));
            }
            StatusCode::TOO_MANY_REQUESTS => {
                // backoff if rate limited, 5s default
                let retry_after = response
                    .headers()
                    .get("Retry-After")
                    .and_then(|header| header.to_str().ok())
                    .and_then(|value| value.parse::<u64>().ok())
                    .unwrap_or(5);
                *self.backoff.write().await = SystemTime::now() + Duration::from_secs(retry_after);
                return Err(ApiError::ResponseError429);
            }
            _ => return Err(errors::return_response_error(status)),
        }
    }

    // fetches and stores a cached track's audio features
    pub async fn cache_audio_features(&self, db_manager: &DBManager, uri: &str) {
        let Some(spotify_id) = get_id_from_uri(uri) else {
            return;
        };
        match self.audio_features(&spotify_id).await {
            Ok(Some(features)) => {
                if let Err(e) = db_manager.save_audio_features(uri, &features).await {
                    warn!("Failed to save audio features for {}: {}.", uri, e);
                }
            }
            Ok(None) => debug!("ReccoBeats has no audio features for {}.", uri),
            Err(e) => warn!("Failed to get audio features for {}: {}.", uri, e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use warp::Filter;

    fn recorded() -> Value {
        return serde_json::json!({"content": [
            {"id": "834913ac-02bd-4134-a793-ec99eb5309f3",
             "href": "https://open.spotify.com/track/1fJFuvU2ldmeAm5nFIHcPP",
             "isrc": "USMC10110874", "acousticness": 7.15E-4, "danceability": 0.579,
             "energy": 0.928, "instrumentalness": 0.0, "key": 0, "liveness": 0.18,
             "loudness": -4.344, "mode": 1, "speechiness": 0.0411, "tempo": 95.594,
             "valence": 0.882},
            {"id": "50d5433c-969e-4d38-b3da-f23dc5a0cbf8",
             "href": "https://open.spotify.com/track/0EdMqiKs9LKXhspeQhl4RZ",
             "isrc": "GBAHT1901181", "acousticness": 6.44E-4, "danceability": 0.544,
             "energy": 0.892, "instrumentalness": 0.00266, "key": 9, "liveness": 0.0832,
             "loudness": -3.212, "mode": 0, "speechiness": 0.0591, "tempo": 118.005,
             "valence": 0.317}
        ]});
    }

    // mock reccobeats: known ids get the recorded content, "limited" gets a 429
    fn mock_server() -> (String, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&calls);
        let route = warp::path!("v1" / "audio-features")
            .and(warp::query::<HashMap<String, String>>())
            .map(move |q: HashMap<String, String>| {
                counter.fetch_add(1, Ordering::SeqCst);
                let ids = q.get("ids").cloned().unwrap_or_default();
                if ids == "limited" {
                    return warp::reply::with_status(
                        warp::reply::json(&serde_json::json!({})),
                        warp::http::StatusCode::TOO_MANY_REQUESTS,
                    );
                }
                let body = if recorded().to_string().contains(&ids) {
                    recorded()
                } else {
                    serde_json::json!({"content": []})
                };
                warp::reply::with_status(warp::reply::json(&body), warp::http::StatusCode::OK)
            });
        let (addr, server) = warp::serve(route).bind_ephemeral(([127, 0, 0, 1], 0));
        tokio::spawn(server);
        return (format!("http://{}", addr), calls);
    }

    #[tokio::test]
    async fn audio_features_matches_by_spotify_id() {
        let (url, _) = mock_server();
        let recco = ReccoBeatsProxy::new(url);

        let features = recco
            .audio_features("0EdMqiKs9LKXhspeQhl4RZ")
            .await
            .unwrap();
        assert_eq!(features.unwrap()["tempo"], 118.005);
        let features = recco
            .audio_features("1fJFuvU2ldmeAm5nFIHcPP")
            .await
            .unwrap();
        assert_eq!(features.unwrap()["tempo"], 95.594);
        assert_eq!(recco.audio_features("unknown").await.unwrap(), None);
    }

    #[tokio::test]
    async fn rate_limit_sets_backoff_and_skips_calls() {
        let (url, calls) = mock_server();
        let recco = ReccoBeatsProxy::new(url);

        assert!(matches!(
            recco.audio_features("limited").await,
            Err(ApiError::ResponseError429)
        ));
        // within the 5s default backoff: no request is sent
        assert!(matches!(
            recco.audio_features("1fJFuvU2ldmeAm5nFIHcPP").await,
            Err(ApiError::ResponseError429)
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
