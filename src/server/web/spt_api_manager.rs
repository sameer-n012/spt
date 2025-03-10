// use once_cell::sync::OnceCell;
use crate::util::errors::{self, ApiError};
use base64::{engine::general_purpose, Engine};
use rand::Rng;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::iter;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Notify;
use url::Url;

#[derive(Debug, Clone)]
pub struct ApiManager {
    client: Client,
    base_url: String,
    client_id: String,
    client_secret: String,
    access_token: Option<(String, SystemTime)>, // (token, expiry time)
    refresh_token: Option<String>,
    callback_uri: String,
    backoff: SystemTime, // time to start api calls again
    scope: String,
    pub cb_auth_code: Option<String>,
    pub cb_auth_notifier: Arc<Notify>,
}

impl ApiManager {
    pub fn new() -> Self {
        let client_id = env::var("SPT_API_CLIENT_ID").expect("SPT_API_CLIENT_ID must be set");
        let client_secret =
            env::var("SPT_API_CLIENT_SECRET").expect("SPT_API_CLIENT_SECRET must be set");
        let base_url = env::var("SPT_API_BASE_URL").expect("SPT_API_BASE_URL must be set");
        let callback_uri =
            env::var("SERVER_CALLBACK_URL").expect("SERVER_CALLBACK_URL must be set");
        let scope = env::var("SPT_API_SCOPE").expect("SPT_API_SCOPE must be set");

        return ApiManager {
            client: Client::new(),
            base_url,
            client_id,
            client_secret,
            access_token: None,
            refresh_token: None,
            callback_uri,
            backoff: SystemTime::now(),
            scope,
            cb_auth_code: None,
            cb_auth_notifier: Arc::new(Notify::new()),
        };
    }

    pub async fn execute_backoff(&mut self) -> Result<(), ApiError> {
        loop {
            // backoff if necessary
            match self.backoff.duration_since(SystemTime::now()) {
                Ok(duration) => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(duration.as_secs())).await;
                }
                Err(_) => {} // no backoff needed
            }

            // send sample API call
            let url = format!("{}/markets", self.base_url);

            let request = self
                .client
                .get(&url)
                .bearer_auth(&self.access_token.clone().unwrap().0);

            let response = match request.send().await {
                Ok(res) => res,
                Err(_) => return Err(ApiError::BackoffError),
            };
            let status = response.status();

            // check if still rate limited
            match status.as_u16() {
                429 => {
                    if let Some(retry_after) = response
                        .headers()
                        .get("Retry-After")
                        .and_then(|header| header.to_str().ok())
                        .and_then(|value| value.parse::<u64>().ok())
                    {
                        self.backoff = SystemTime::now() + Duration::from_secs(retry_after);
                    } else {
                        self.backoff = SystemTime::now() + Duration::from_secs(5);
                        // default backoff
                    }
                    continue; // Retry after the backoff
                }
                200 => {
                    return Ok(());
                }
                _ => {
                    return Err(errors::return_response_error(status));
                }
            }
        }
    }

    fn gen_random_state(&self, len: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();
        let one_char = || CHARSET[rng.gen_range(0..CHARSET.len())] as char;
        iter::repeat_with(one_char).take(len).collect()
    }

    fn gen_challenge(&self, state: &String) -> String {
        let mut sha = Sha256::new();
        sha.update(state.as_bytes());

        // let b64_engine = engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);
        // return b64_engine.encode(sha.finalize());
        return general_purpose::URL_SAFE_NO_PAD.encode(sha.finalize());
    }

    async fn auth(&mut self) -> Result<StatusCode, ApiError> {
        // self.execute_backoff().await?;

        // generate state and challenge
        let state = self.gen_random_state(64);
        let challenge = self.gen_challenge(&state);

        // request parameters
        let params = vec![
            ("response_type", "code"),
            ("client_id", &self.client_id),
            ("redirect_uri", &self.callback_uri),
            ("scope", &self.scope),
            ("code_challenge", &challenge),
            ("code_challenge_method", "S256"),
        ];

        let url = match Url::parse_with_params("https://accounts.spotify.com/authorize", &params) {
            Ok(parsed_url) => Into::<String>::into(parsed_url),
            Err(_) => return Err(ApiError::RequestError),
        };

        // Start callback server before opening the browser
        // let server_task = tokio::spawn(start_callback_server(8081));

        if let Err(_) = open::that(url) {
            return Err(ApiError::BrowserError);
        }

        println!("here 0");

        // Await server response
        // let code = match server_task.await {
        //     Ok(code) => code,
        //     Err(_) => Err(ApiError::InternalServerError),
        // };
        let code = {
            self.cb_auth_notifier.notified().await;
            self.cb_auth_code
                .take()
                .ok_or(ApiError::InternalServerError)?
        };

        println!("here 0.5");

        // request parameters
        let params = [
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", &self.callback_uri),
            ("code_verifier", &state),
            ("client_id", &self.client_id),
        ];

        // send request
        let request = self
            .client
            .post("https://accounts.spotify.com/api/token")
            .form(&params);

        let response = match request.send().await {
            Ok(res) => res,
            Err(_) => return Err(ApiError::RequestError),
        };

        self.cb_auth_code = None;

        let status = response.status();
        println!("s {}", status);

        if status.is_success() {
            // parse response json
            let json = match response.json::<Value>().await {
                Ok(data) => data,
                Err(_) => {
                    println!("here 1");

                    return Err(ApiError::ResponseParseError);
                }
            };

            // update access token, expiry time, and refresh token
            let access_token = match json["access_token"].as_str() {
                Some(token) => token.to_string(),
                None => return Err(ApiError::ResponseDataError),
            };

            let expires_in = match json["expires_in"].as_u64() {
                Some(exp) => exp,
                None => return Err(ApiError::ResponseDataError),
            };

            let refresh_token = match json["refresh_token"].as_str() {
                Some(token) => token.to_string(),
                None => return Err(ApiError::ResponseDataError),
            };

            let duration = SystemTime::now() + Duration::new(expires_in, 0);

            self.access_token = Some((access_token, duration));
            self.refresh_token = Some(refresh_token);

            return Ok(status);
        }

        Err(errors::return_response_error(status))
    }

    pub async fn validate_auth(&mut self) -> Result<StatusCode, ApiError> {
        if self.access_token.is_none() || self.access_token.clone().unwrap().1 < SystemTime::now() {
            if self.refresh_token.is_none() {
                return self.auth().await;
            } else {
                return self.reauth().await;
            }
        }

        return Ok(StatusCode::OK);
    }

    // Method for refreshing the Spotify API token
    pub async fn reauth(&mut self) -> Result<StatusCode, ApiError> {
        // self.execute_backoff().await?;

        // ensure refresh token is present
        let refresh_token = match &self.refresh_token {
            Some(token) => token,
            None => {
                return self.auth().await;
            }
        };

        // request parameters
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.client_id),
        ];

        // send request
        let request = self
            .client
            .post("https://accounts.spotify.com/api/token")
            .form(&params);

        let response = match request.send().await {
            Ok(res) => res,
            Err(_) => return Err(ApiError::RequestError),
        };

        let status = response.status();

        // success response
        if status.is_success() {
            // parse json response
            let json = match response.json::<Value>().await {
                Ok(data) => data,
                Err(_) => {
                    println!("here 2");

                    return Err(ApiError::ResponseParseError);
                }
            };

            // update access token and expiry time
            let access_token = match json["access_token"].as_str() {
                Some(token) => token.to_string(),
                None => return Err(ApiError::ResponseDataError),
            };

            let expires_in = match json["expires_in"].as_u64() {
                Some(exp) => exp,
                None => return Err(ApiError::ResponseDataError),
            };

            let duration = SystemTime::now() + Duration::new(expires_in, 0);

            self.access_token = Some((access_token, duration));

            // update refresh token if new one is provided
            if let Some(refresh_token) = json["refresh_token"].as_str() {
                self.refresh_token = Some(refresh_token.to_string());
            }

            return Ok(status);
        }

        // non-success response
        return Err(errors::return_response_error(status));
    }

    // Method for sending GET requests to the Spotify API
    pub async fn get(
        &mut self,
        endpoint: &str,
        params: Option<HashMap<&str, &str>>,
    ) -> Result<(StatusCode, Value), ApiError> {
        // check if access token is valid, if not, auth/reauth
        println!("{}", endpoint);
        self.validate_auth().await?;

        // backoff
        println!("{}", endpoint);
        self.execute_backoff().await?;

        // construct and send request
        let url = format!("{}/{}", self.base_url, endpoint);

        let request = self
            .client
            .get(&url)
            .bearer_auth(
                &self
                    .access_token
                    .clone()
                    .ok_or_else(|| ApiError::NoAccessToken)?
                    .0,
            )
            .query(&params.unwrap_or_default());

        let response = match request.send().await {
            Ok(res) => res,
            Err(_) => return Err(ApiError::RequestError),
        };

        let status = response.status();

        // match status code
        match status.as_u16() {
            200 => {
                let json = match response.json::<Value>().await {
                    Ok(data) => data,
                    Err(_) => {
                        println!("here 3");

                        return Err(ApiError::ResponseParseError);
                    }
                };
                return Ok((status, json));
            }
            204 => Ok((status, serde_json::json!({}))),
            401 => {
                // refresh access token if needed
                self.access_token = None;
                self.validate_auth().await?;
                return Err(ApiError::InvalidAccessToken);
            }
            429 => {
                // backoff if rate limited
                self.backoff = SystemTime::now() + Duration::from_secs(5);
                return Err(ApiError::ResponseError429);
            }
            _ => Err(errors::return_response_error(status)),
        }
    }

    // Method for sending POST requests to the Spotify API
    pub async fn post(
        &mut self,
        endpoint: &str,
        body: Option<Value>,
    ) -> Result<(StatusCode, Value), ApiError> {
        // backoff
        self.execute_backoff().await?;

        // check if access token is valid, if not, auth/reauth
        self.validate_auth().await?;

        // construct and send request
        let url = format!("{}/{}", self.base_url, endpoint);

        let request = self
            .client
            .post(&url)
            .bearer_auth(
                &self
                    .access_token
                    .clone()
                    .ok_or_else(|| ApiError::NoAccessToken)?
                    .0,
            )
            .json(&body.unwrap_or_default());

        let response = match request.send().await {
            Ok(res) => res,
            Err(_) => return Err(ApiError::RequestError),
        };

        let status = response.status();

        // match status code
        match status.as_u16() {
            200 => {
                let json = match response.json::<Value>().await {
                    Ok(data) => data,
                    Err(_) => {
                        return Err(ApiError::ResponseParseError);
                    }
                };
                return Ok((status, json));
            }
            204 => Ok((status, serde_json::json!({}))),
            401 => {
                // refresh access token if needed
                self.access_token = None;
                self.validate_auth().await?;
                return Err(ApiError::InvalidAccessToken);
            }
            429 => {
                // backoff if rate limited
                self.backoff = SystemTime::now() + Duration::from_secs(5);
                return Err(ApiError::ResponseError429);
            }
            _ => Err(errors::return_response_error(status)),
        }
    }

    // Method for sending PUT requests to the Spotify API
    pub async fn put(
        &mut self,
        endpoint: &str,
        body: Option<Value>,
    ) -> Result<(StatusCode, Value), ApiError> {
        // backoff
        self.execute_backoff().await?;

        // check if access token is valid, if not, auth/reauth
        self.validate_auth().await?;

        // construct and send request
        let url = format!("{}/{}", self.base_url, endpoint);

        let request = self
            .client
            .put(&url)
            .bearer_auth(
                &self
                    .access_token
                    .clone()
                    .ok_or_else(|| ApiError::NoAccessToken)?
                    .0,
            )
            .json(&body.unwrap_or_default());

        let response = match request.send().await {
            Ok(res) => res,
            Err(_) => return Err(ApiError::RequestError),
        };

        let status = response.status();

        // match status code
        match status.as_u16() {
            200 => {
                let json = match response.json::<Value>().await {
                    Ok(data) => data,
                    Err(_) => {
                        return Err(ApiError::ResponseParseError);
                    }
                };
                return Ok((status, json));
            }
            204 => Ok((status, serde_json::json!({}))),
            401 => {
                // refresh access token if needed
                self.access_token = None;
                self.validate_auth().await?;
                return Err(ApiError::InvalidAccessToken);
            }
            429 => {
                // backoff if rate limited
                self.backoff = SystemTime::now() + Duration::from_secs(5);
                return Err(ApiError::ResponseError429);
            }
            _ => Err(errors::return_response_error(status)),
        }
    }

    // Example method to get devices (for demonstration purposes)
    // pub async fn get_devices(&self) -> Result<(StatusCode, serde_json::Value), Error> {
    //     self.get("me/player/devices", None).await
    // }
}
