// use once_cell::sync::OnceCell;
use crate::server::web::server::start_server;
use crate::util::errors::{self, return_response_code, ApiError};
use log::{debug, error, info, warn};
use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::time::Duration;

#[derive(Debug)]
pub struct ApiProxy {
    client: Client,
    client_id: Option<u64>,
    base_url: String,
    server_port: u16,
    server_timeout: Duration,
    max_server_retries: u8,
}

impl ApiProxy {
    pub fn new() -> Self {
        let base_url = env::var("SERVER_BASE_URL").expect("SERVER_BASE_URL must be set");
        let max_server_retries = env::var("MAX_SERVER_RETRIES")
            .expect("MAX_SERVER_RETRIES must be set")
            .parse::<u8>()
            .unwrap();
        let server_port = env::var("SERVER_PORT")
            .expect("SERVER_PORT must be set")
            .parse::<u16>()
            .unwrap();
        let server_timeout = Duration::from_secs(
            env::var("SERVER_TIMEOUT_SECONDS")
                .expect("SERVER_TIMEOUT_SECONDS must be set")
                .parse::<u64>()
                .unwrap(),
        );

        let api_manager = ApiProxy {
            client: Client::new(),
            client_id: None,
            base_url,
            server_port,
            server_timeout,
            max_server_retries,
        };

        return api_manager;
    }

    pub async fn setup(&mut self) -> Result<(), ApiError> {
        // This method is  used to perform any setup required for the API manager

        debug!("Setting up client API proxy.");

        self.check_server(0).await?;

        let client_id =
            ApiProxy::get_client_id(&self.client, format!("{}/{}", self.base_url, "init")).await;

        match client_id {
            Ok(id) => match id.trim_matches('"').parse::<u64>() {
                Ok(parsed_id) if parsed_id > 0 => {
                    info!("Client recieved client_id {} from server.", parsed_id);
                    self.client_id = Some(parsed_id);
                    return Ok(());
                }
                _ => {
                    error!("Client recieved invalid client_id from server.");
                    return Err(ApiError::ResponseDataError);
                }
            },
            Err(e) => {
                error!(
                    "Client requested client_id and received response from server with status {}.",
                    return_response_code(e)
                );
                return Err(ApiError::InternalServerError);
            }
        }
    }

    async fn get_client_id(client: &Client, url: String) -> Result<String, ApiError> {
        debug!("Client requesting client_id from server.");

        let response = client.get(&url).send().await;
        if response.is_ok() {
            let response = response.unwrap();
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
                    return Ok(json["client_id"].to_string());
                }
                _ => Err(errors::return_response_error(status)),
            }
        } else {
            return Err(ApiError::InternalServerError);
        }
    }

    // Method to check if server is running, and if not, start it
    pub async fn check_server(&self, retry: u8) -> Result<(), ApiError> {
        let url = format!("{}/ping", self.base_url);

        let response = self.client.get(&url).send().await;

        if response.is_ok() && response.unwrap().status().as_u16() == 200 {
            info!("Client found server running.");
            return Ok(());
        } else {
            if retry >= self.max_server_retries {
                return Err(ApiError::InternalServerError);
            }
            info!("Client found server down, attempting to start it.");
            let _ = start_server(self.server_port, self.server_timeout).await;
            // self.check_server(retry + 1).await?;
            return Ok(());
        }
    }

    // Method for sending GET requests to the Spotify API
    pub async fn get(
        &self,
        endpoint: &str,
        params: Option<HashMap<&str, &str>>,
    ) -> Result<(StatusCode, Value), ApiError> {
        // check if server is up, if not, start it
        // self.check_server(0).await?;

        if self.client_id.is_none() {
            return Err(ApiError::InvalidAccessToken);
        }

        // construct and send request
        let url = format!(
            "{}/{}?client_id={}",
            self.base_url,
            endpoint,
            self.client_id.unwrap()
        );

        let request = self.client.get(&url).query(&params.unwrap_or_default());

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
            _ => Err(errors::return_response_error(status)),
        }
    }

    // Method for sending POST requests to the Spotify API
    pub async fn post(
        &self,
        endpoint: &str,
        body: Option<Value>,
    ) -> Result<(StatusCode, Value), ApiError> {
        // check if server is up, if not, start it
        // self.check_server(0).await?;

        if self.client_id.is_none() {
            return Err(ApiError::InvalidAccessToken);
        }

        // construct and send request
        let url = format!(
            "{}/{}?client_id={}",
            self.base_url,
            endpoint,
            self.client_id.unwrap()
        );

        let request = self.client.post(&url).json(&body.unwrap_or_default());

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
            _ => Err(errors::return_response_error(status)),
        }
    }

    // Method for sending PUT requests to the Spotify API
    pub async fn put(
        &self,
        endpoint: &str,
        body: Option<Value>,
    ) -> Result<(StatusCode, Value), ApiError> {
        // check if server is up, if not, start it
        // self.check_server(0).await?;

        if self.client_id.is_none() {
            return Err(ApiError::InvalidAccessToken);
        }

        // construct and send request
        let url = format!(
            "{}/{}?client_id={}",
            self.base_url,
            endpoint,
            self.client_id.unwrap()
        );

        let request = self.client.put(&url).json(&body.unwrap_or_default());

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
            _ => Err(errors::return_response_error(status)),
        }
    }

    // Example method to get devices (for demonstration purposes)
    // pub async fn get_devices(&self) -> Result<(StatusCode, serde_json::Value), Error> {
    //     self.get("me/player/devices", None).await
    // }
}
