use crate::util::errors::{self, ApiError};
use base64::{engine::general_purpose, Engine};
use log::{debug, error, info, warn};
use rand::Rng;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::iter;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{Notify, RwLock};
use url::Url;

#[derive(Debug, Clone)]
struct AuthInfo {
    // these should be locked together
    access_token: Option<(String, SystemTime)>, // (token, expiry time)
    refresh_token: Option<String>,
    cb_auth_code: Option<String>,
}

#[derive(Debug)]
pub struct ApiProxy {
    client: Client,

    application_id: String,
    // application_secret: String,
    scope: String,

    base_url: String,
    callback_url: String,
    backoff: RwLock<SystemTime>, // time to start api calls again

    user_client_id: u64, // unique client id for each user, each user gets their own ApiProxy
    auth_info: RwLock<AuthInfo>,
    pub cb_auth_notifier: Arc<Notify>,
}

impl ApiProxy {
    pub fn new(user_client_id: u64) -> Self {
        let client_id = env::var("SPT_API_CLIENT_ID").expect("SPT_API_CLIENT_ID must be set");
        // let client_secret =
        //     env::var("SPT_API_CLIENT_SECRET").expect("SPT_API_CLIENT_SECRET must be set");
        let base_url = env::var("SPT_API_BASE_URL").expect("SPT_API_BASE_URL must be set");
        let callback_url =
            env::var("SERVER_CALLBACK_URL").expect("SERVER_CALLBACK_URL must be set");
        let scope = env::var("SPT_API_SCOPE").expect("SPT_API_SCOPE must be set");

        return ApiProxy {
            client: Client::new(),

            application_id: client_id,
            // application_secret: client_secret,
            scope,

            base_url,
            callback_url,
            backoff: RwLock::new(SystemTime::now()),

            user_client_id,
            auth_info: RwLock::new(AuthInfo {
                access_token: None,
                refresh_token: None,
                cb_auth_code: None,
            }),
            cb_auth_notifier: Arc::new(Notify::new()),
        };
    }

    pub async fn set_cb_auth_code(&self, code: String) {
        let mut auth_info = self.auth_info.write().await;
        auth_info.cb_auth_code = Some(code);
        self.cb_auth_notifier.notify_one();
    }

    pub async fn unset_cb_auth_code(&self) {
        let mut auth_info = self.auth_info.write().await;
        auth_info.cb_auth_code = None;
    }

    pub async fn execute_backoff(&self) -> Result<(), ApiError> {
        // loop {
        // get backoff lock here
        let backoff = self.backoff.read().await;

        // backoff if necessary
        match backoff.duration_since(SystemTime::now()) {
            Ok(duration) => {
                debug!("Client {} entering backoff loop.", self.user_client_id);

                tokio::time::sleep(tokio::time::Duration::from_secs(duration.as_secs())).await;

                return Ok(());
            }
            Err(_) => {
                return Ok(()); // no backoff needed
            }
        }

        // send sample API call
        // let url = format!("{}/markets", self.base_url);

        // let auth_info = self.auth_info.read().await;

        // let request = self
        //     .client
        //     .get(&url)
        //     .bearer_auth(&auth_info.access_token.clone().unwrap().0);

        // let response = match request.send().await {
        //     Ok(res) => res,
        //     Err(_) => return Err(ApiError::BackoffError),
        // };
        // let status = response.status();

        // // check if still rate limited
        // match status.as_u16() {
        //     429 => {
        //         if let Some(retry_after) = response
        //             .headers()
        //             .get("Retry-After")
        //             .and_then(|header| header.to_str().ok())
        //             .and_then(|value| value.parse::<u64>().ok())
        //         {
        //             *backoff = SystemTime::now() + Duration::from_secs(retry_after);
        //         } else {
        //             *backoff = SystemTime::now() + Duration::from_secs(5);
        //             // default backoff
        //         }
        //         continue; // Retry after the backoff
        //     }
        //     200 => {
        //         return Ok(());
        //     }
        //     _ => {
        //         return Err(errors::return_response_error(status));
        //     }
        // }
        // }
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

    async fn auth(&self) -> Result<StatusCode, ApiError> {
        // self.execute_backoff().await?;

        info!(
            "Client {} attempting to aunthenticate.",
            self.user_client_id
        );

        // generate state and challenge
        let state = self.gen_random_state(64);
        let challenge = self.gen_challenge(&state);

        let sent_state = self.user_client_id.to_string();

        // request parameters
        let params = vec![
            ("response_type", "code"),
            ("client_id", &self.application_id),
            ("redirect_uri", &self.callback_url),
            ("scope", &self.scope),
            ("code_challenge", &challenge),
            ("code_challenge_method", "S256"),
            ("state", &sent_state),
        ];

        let url = match Url::parse_with_params("https://accounts.spotify.com/authorize", &params) {
            Ok(parsed_url) => Into::<String>::into(parsed_url),
            Err(_) => return Err(ApiError::RequestError),
        };

        if let Err(_) = open::that(url) {
            return Err(ApiError::BrowserError);
        }

        let code: String = {
            self.cb_auth_notifier.notified().await;

            let mut auth_info = self.auth_info.write().await;

            auth_info
                .cb_auth_code
                .take()
                .ok_or(ApiError::InternalServerError)?
        };

        debug!(
            "Client {} received callback authentication code.",
            self.user_client_id
        );

        // request parameters
        let params = [
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", &self.callback_url),
            ("code_verifier", &state),
            ("client_id", &self.application_id),
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

        // shouldn't need to unset since we take() earlier
        // self.unset_cb_auth_code().await;

        let status = response.status();

        if status.is_success() {
            // parse response json
            let json = match response.json::<Value>().await {
                Ok(data) => data,
                Err(_) => {
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

            {
                let mut auth_info = self.auth_info.write().await;

                auth_info.access_token = Some((access_token, duration));
                auth_info.refresh_token = Some(refresh_token);
            }

            info!(
                "Client {} successfully aunthenticated.",
                self.user_client_id
            );

            return Ok(status);
        }

        error!("Client {} failed to authenticate.", self.user_client_id);

        {
            let mut auth_info = self.auth_info.write().await;
            auth_info.access_token = None;
            auth_info.refresh_token = None;
        }
        self.unset_cb_auth_code().await;

        Err(errors::return_response_error(status))
    }

    pub async fn validate_auth(&self) -> Result<StatusCode, ApiError> {
        let (at, rt) = {
            let auth_info = self.auth_info.read().await;
            (
                auth_info.access_token.clone(),
                auth_info.refresh_token.clone(),
            )
        };

        if at.is_none() || at.unwrap().1 < SystemTime::now() {
            if rt.is_none() {
                return self.auth().await;
            } else {
                match self.reauth().await {
                    Ok(status) => return Ok(status),
                    Err(_) => {
                        return self.auth().await;
                    }
                }
            }
        }

        return Ok(StatusCode::OK);
    }

    // Method for refreshing the Spotify API token
    pub async fn reauth(&self) -> Result<StatusCode, ApiError> {
        // self.execute_backoff().await?;

        info!(
            "Client {} attempting to reaunthenticate.",
            self.user_client_id
        );

        // ensure refresh token is present
        let refresh_token = {
            let auth_info = self.auth_info.read().await;
            auth_info.refresh_token.clone()
        };
        let refresh_token = match refresh_token {
            Some(token) => token,
            None => return self.auth().await,
        };

        // request parameters
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", &refresh_token),
            ("client_id", &self.application_id),
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

            {
                let mut auth_info = self.auth_info.write().await;

                auth_info.access_token = Some((access_token, duration));
                auth_info.refresh_token = Some(refresh_token);

                // update refresh token if new one is provided
                if let Some(refresh_token) = json["refresh_token"].as_str() {
                    auth_info.refresh_token = Some(refresh_token.to_string());
                }
            }

            info!(
                "Client {} successfully reaunthenticated.",
                self.user_client_id
            );

            return Ok(status);
        }

        // non-success response
        warn!("Client {} failed to reaunthenticate.", self.user_client_id);

        {
            let mut auth_info = self.auth_info.write().await;
            auth_info.access_token = None;
            auth_info.refresh_token = None;
        }
        self.unset_cb_auth_code().await;

        return Err(errors::return_response_error(status));
    }

    // Method for sending GET requests to the Spotify API
    pub async fn get(
        &self,
        endpoint: &str,
        params: Option<HashMap<&str, &str>>,
    ) -> Result<(StatusCode, Value), ApiError> {
        // check if access token is valid, if not, auth/reauth
        self.validate_auth().await?;

        // backoff
        self.execute_backoff().await?;

        // construct and send request
        let url = format!("{}/{}", self.base_url, endpoint);

        let access_token = {
            let auth_info = self.auth_info.read().await;
            auth_info.access_token.clone()
        };

        info!(
            "Client {} sending request to {}/{}.",
            self.user_client_id, self.base_url, endpoint
        );

        let request = self
            .client
            .get(&url)
            .bearer_auth(access_token.ok_or_else(|| ApiError::NoAccessToken)?.0)
            .query(&params.unwrap_or_default());

        let response = match request.send().await {
            Ok(res) => res,
            Err(_) => return Err(ApiError::RequestError),
        };

        let status = response.status();

        if status.as_u16() == 200 || status.as_u16() == 204 {
            info!(
                "Client {} received response from {}/{} with status {}.",
                self.user_client_id, self.base_url, endpoint, status
            );
        } else {
            warn!(
                "Client {} received response from {}/{} with status {}.",
                self.user_client_id, self.base_url, endpoint, status
            );
        }

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
                {
                    let mut auth_info = self.auth_info.write().await;
                    auth_info.access_token = None;
                }
                self.validate_auth().await?;
                return Err(ApiError::InvalidAccessToken);
            }
            429 => {
                // backoff if rate limited
                let mut backoff = self.backoff.write().await;
                if let Some(retry_after) = response
                    .headers()
                    .get("Retry-After")
                    .and_then(|header| header.to_str().ok())
                    .and_then(|value| value.parse::<u64>().ok())
                {
                    *backoff = SystemTime::now() + Duration::from_secs(retry_after);
                } else {
                    *backoff = SystemTime::now() + Duration::from_secs(5);
                    // default backoff
                }

                return Err(ApiError::ResponseError429);
            }
            _ => Err(errors::return_response_error(status)),
        }
    }

    // Method for sending POST requests to the Spotify API
    pub async fn post(
        &self,
        endpoint: &str,
        body: Option<Value>,
    ) -> Result<(StatusCode, Value), ApiError> {
        // backoff
        self.execute_backoff().await?;

        // check if access token is valid, if not, auth/reauth
        self.validate_auth().await?;

        // construct and send request
        let url = format!("{}/{}", self.base_url, endpoint);

        let access_token = {
            let auth_info = self.auth_info.read().await;
            auth_info.access_token.clone()
        };

        info!(
            "Client {} sending request to {}/{}.",
            self.user_client_id, self.base_url, endpoint
        );

        let request = self
            .client
            .post(&url)
            .bearer_auth(access_token.ok_or_else(|| ApiError::NoAccessToken)?.0)
            .json(&body.unwrap_or_default());

        let response = match request.send().await {
            Ok(res) => res,
            Err(_) => return Err(ApiError::RequestError),
        };

        let status = response.status();

        if status.as_u16() == 200 || status.as_u16() == 204 {
            info!(
                "Client {} received response from {}/{} with status {}.",
                self.user_client_id, self.base_url, endpoint, status
            );
        } else {
            warn!(
                "Client {} received response from {}/{} with status {}.",
                self.user_client_id, self.base_url, endpoint, status
            );
        }

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
                {
                    let mut auth_info = self.auth_info.write().await;
                    auth_info.access_token = None;
                }
                self.validate_auth().await?;
                return Err(ApiError::InvalidAccessToken);
            }
            429 => {
                // backoff if rate limited
                let mut backoff = self.backoff.write().await;
                if let Some(retry_after) = response
                    .headers()
                    .get("Retry-After")
                    .and_then(|header| header.to_str().ok())
                    .and_then(|value| value.parse::<u64>().ok())
                {
                    *backoff = SystemTime::now() + Duration::from_secs(retry_after);
                } else {
                    *backoff = SystemTime::now() + Duration::from_secs(5);
                    // default backoff
                }

                return Err(ApiError::ResponseError429);
            }
            _ => Err(errors::return_response_error(status)),
        }
    }

    // Method for sending PUT requests to the Spotify API
    pub async fn put(
        &self,
        endpoint: &str,
        body: Option<Value>,
    ) -> Result<(StatusCode, Value), ApiError> {
        // backoff
        self.execute_backoff().await?;

        // check if access token is valid, if not, auth/reauth
        self.validate_auth().await?;

        // construct and send request
        let url = format!("{}/{}", self.base_url, endpoint);

        let access_token = {
            let auth_info = self.auth_info.read().await;
            auth_info.access_token.clone()
        };

        info!(
            "Client {} sending request to {}/{}.",
            self.user_client_id, self.base_url, endpoint
        );

        let request = self
            .client
            .put(&url)
            .bearer_auth(access_token.ok_or_else(|| ApiError::NoAccessToken)?.0)
            .json(&body.unwrap_or_default());

        let response = match request.send().await {
            Ok(res) => res,
            Err(_) => return Err(ApiError::RequestError),
        };

        let status = response.status();

        if status.as_u16() == 200 || status.as_u16() == 204 {
            info!(
                "Client {} received response from {}/{} with status {}.",
                self.user_client_id, self.base_url, endpoint, status
            );
        } else {
            warn!(
                "Client {} received response from {}/{} with status {}.",
                self.user_client_id, self.base_url, endpoint, status
            );
        }

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
                {
                    let mut auth_info = self.auth_info.write().await;
                    auth_info.access_token = None;
                }
                self.validate_auth().await?;
                return Err(ApiError::InvalidAccessToken);
            }
            429 => {
                // backoff if rate limited
                let mut backoff = self.backoff.write().await;
                if let Some(retry_after) = response
                    .headers()
                    .get("Retry-After")
                    .and_then(|header| header.to_str().ok())
                    .and_then(|value| value.parse::<u64>().ok())
                {
                    *backoff = SystemTime::now() + Duration::from_secs(retry_after);
                } else {
                    *backoff = SystemTime::now() + Duration::from_secs(5);
                    // default backoff
                }

                return Err(ApiError::ResponseError429);
            }
            _ => Err(errors::return_response_error(status)),
        }
    }

    // Method for sending DELETE requests to the Spotify API
    pub async fn delete(
        &self,
        endpoint: &str,
        body: Option<Value>,
    ) -> Result<(StatusCode, Value), ApiError> {
        // backoff
        self.execute_backoff().await?;

        // check if access token is valid, if not, auth/reauth
        self.validate_auth().await?;

        // construct and send request
        let url = format!("{}/{}", self.base_url, endpoint);

        let access_token = {
            let auth_info = self.auth_info.read().await;
            auth_info.access_token.clone()
        };

        info!(
            "Client {} sending request to {}/{}.",
            self.user_client_id, self.base_url, endpoint
        );

        let request = self
            .client
            .delete(&url)
            .bearer_auth(access_token.ok_or_else(|| ApiError::NoAccessToken)?.0)
            .json(&body.unwrap_or_default());

        let response = match request.send().await {
            Ok(res) => res,
            Err(_) => return Err(ApiError::RequestError),
        };

        let status = response.status();

        if status.as_u16() == 200 || status.as_u16() == 204 {
            info!(
                "Client {} received response from {}/{} with status {}.",
                self.user_client_id, self.base_url, endpoint, status
            );
        } else {
            warn!(
                "Client {} received response from {}/{} with status {}.",
                self.user_client_id, self.base_url, endpoint, status
            );
        }

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
                {
                    let mut auth_info = self.auth_info.write().await;
                    auth_info.access_token = None;
                }
                self.validate_auth().await?;
                return Err(ApiError::InvalidAccessToken);
            }
            429 => {
                // backoff if rate limited
                let mut backoff = self.backoff.write().await;
                if let Some(retry_after) = response
                    .headers()
                    .get("Retry-After")
                    .and_then(|header| header.to_str().ok())
                    .and_then(|value| value.parse::<u64>().ok())
                {
                    *backoff = SystemTime::now() + Duration::from_secs(retry_after);
                } else {
                    *backoff = SystemTime::now() + Duration::from_secs(5);
                    // default backoff
                }

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
