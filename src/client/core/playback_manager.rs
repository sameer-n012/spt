use crate::client::local_api_proxy::ApiProxy;
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug)]
pub struct PlaybackManager<'a> {
    curr_device_name: Option<String>,
    device_list: HashMap<String, String>, // Maps device names to IDs
    api_manager: &'a mut ApiProxy,
}

impl<'a> PlaybackManager<'a> {
    pub fn new(api_manager: &'a mut ApiProxy) -> Self {
        return PlaybackManager {
            curr_device_name: None,
            device_list: HashMap::new(),
            api_manager,
        };
    }

    pub async fn now(&self, human_readable: bool) -> Option<String> {
        let res = self
            .api_manager
            .get("api/spt-fwd/me/player/currently-playing", None)
            .await;

        if res.is_ok() {
            let (status, json) = res.unwrap();
            if status.as_u16() == 200 {
                if human_readable {
                    return Some(format!(
                        "{} - {} by {}.",
                        json["item"]["name"],
                        json["item"]["album"]["name"],
                        json["item"]["artists"][0]["name"],
                    ));
                } else {
                    if let Some(uri) = json["item"]["uri"].as_str() {
                        return Some(uri.to_string() + "\n");
                    }
                }
            } else if status.as_u16() == 204 {
                return Some("No track currently playing.".to_string());
            } else if status.as_u16() == 401 || status.as_u16() == 403 {
                return Some("User not authorized".to_string());
            } else if status.as_u16() == 429 {
                return Some("Rate limit exceeded".to_string());
            } else {
                return Some(format!("Error: {}", status));
            }
        } else if res.is_err() {
            return Some(format!("Error: {}", res.unwrap_err()));
        }

        return Some("Unknown error.".to_string());
    }

    pub async fn play(&self) -> Option<String> {
        let res = self
            .api_manager
            .put("api/spt-fwd/me/player/play", None, None)
            .await;

        if res.is_ok() {
            let (status, _) = res.unwrap();
            if status.as_u16() == 200 {
                return Some("Now Playing.".to_string());
            } else if status.as_u16() == 204 {
                return Some("No track currently playing.".to_string());
            } else if status.as_u16() == 401 || status.as_u16() == 403 {
                return Some("User not authorized".to_string());
            } else if status.as_u16() == 429 {
                return Some("Rate limit exceeded".to_string());
            } else {
                return Some(format!("Error: {}", status));
            }
        } else if res.is_err() {
            return Some(format!("Error: {}", res.unwrap_err()));
        }

        return Some("Unknown error.".to_string());
    }

    pub async fn pause(&self) -> Option<String> {
        let res = self
            .api_manager
            .put("api/spt-fwd/me/player/pause", None, None)
            .await;

        if res.is_ok() {
            let (status, _) = res.unwrap();
            if status.as_u16() == 200 {
                return Some("Now Paused.".to_string());
            } else if status.as_u16() == 204 {
                return Some("No track currently playing.".to_string());
            } else if status.as_u16() == 401 || status.as_u16() == 403 {
                return Some("User not authorized".to_string());
            } else if status.as_u16() == 429 {
                return Some("Rate limit exceeded".to_string());
            } else {
                return Some(format!("Error: {}", status));
            }
        } else if res.is_err() {
            return Some(format!("Error: {}", res.unwrap_err()));
        }

        return Some("Unknown error.".to_string());
    }

    pub async fn next(&self, n: u8) -> Option<String> {
        let mut results = Vec::with_capacity(n as usize);

        for _ in 0..n {
            let res = self
                .api_manager
                .post("api/spt-fwd/me/player/next", None, None)
                .await;
            results.push(res);
        }

        let mut output = Vec::new();

        let all_failed = results
            .iter()
            .filter(|res| match res {
                Ok((status, _)) => match status.as_u16() {
                    200 => false,
                    204 => false,
                    401 | 403 => {
                        output.push("User not authorized".to_string());
                        true
                    }
                    429 => {
                        output.push("Rate limit exceeded".to_string());
                        true
                    }
                    _ => {
                        output.push(format!("Error: {}", status));
                        true
                    }
                },
                Err(err) => {
                    output.push(format!("Error: {}", err));
                    true
                }
            })
            .count() as u8;

        output.insert(
            0,
            format!(
                "Skipped {} track{}.",
                n - all_failed,
                if n - all_failed > 1 { "s" } else { "" }
            ),
        );

        return Some(output.join("\n"));
    }

    pub async fn previous(&self, n: u8) -> Option<String> {
        let mut results = Vec::with_capacity(n as usize);

        for _ in 0..n {
            let res = self
                .api_manager
                .post("api/spt-fwd/me/player/previous", None, None)
                .await;
            results.push(res);
        }

        let mut output = Vec::new();

        let all_failed = results
            .iter()
            .filter(|res| match res {
                Ok((status, _)) => match status.as_u16() {
                    200 => false,
                    204 => false,
                    401 | 403 => {
                        output.push("User not authorized".to_string());
                        true
                    }
                    429 => {
                        output.push("Rate limit exceeded".to_string());
                        true
                    }
                    _ => {
                        output.push(format!("Error: {}", status));
                        true
                    }
                },
                Err(err) => {
                    output.push(format!("Error: {}", err));
                    true
                }
            })
            .count() as u8;

        output.insert(
            0,
            format!(
                "Rewinded {} track{}.",
                n - all_failed,
                if n - all_failed > 1 { "s" } else { "" }
            ),
        );

        return Some(output.join("\n"));
    }

    pub async fn set_volume(&self, level: u8) -> Option<String> {
        let mut params = HashMap::new();
        params.insert("volume_percent".to_string(), level.to_string());

        let res = self
            .api_manager
            .put("api/spt-fwd/me/player/volume", None, Some(params))
            .await;

        if res.is_ok() {
            let (status, _) = res.unwrap();
            if status.as_u16() == 200 || status.as_u16() == 204 {
                return Some(format!("Volume set to {}.", level));
            } else if status.as_u16() == 401 || status.as_u16() == 403 {
                return Some("User not authorized".to_string());
            } else if status.as_u16() == 429 {
                return Some("Rate limit exceeded".to_string());
            } else {
                return Some(format!("Error: {}", status));
            }
        } else if res.is_err() {
            return Some(format!("Error: {}", res.unwrap_err()));
        }

        return Some("Unknown error.".to_string());
    }

    pub async fn get_volume(&self) -> Option<String> {
        let res = self
            .api_manager
            .get("api/spt-fwd/me/player/devices", None)
            .await;

        if res.is_ok() {
            let (status, json) = res.unwrap();
            if status.as_u16() == 200 {
                return Some(format!(
                    "Volume: {}% ({})",
                    json["device"]["volume_percent"].to_string(),
                    json["device"]["name"],
                ));
            } else if status.as_u16() == 204 {
                return Some("No devices found.".to_string());
            } else if status.as_u16() == 401 || status.as_u16() == 403 {
                return Some("User not authorized".to_string());
            } else if status.as_u16() == 429 {
                return Some("Rate limit exceeded".to_string());
            }
        } else if res.is_err() {
            return Some(format!("Error: {}", res.unwrap_err()));
        }

        return Some("Unknown error.".to_string());
    }

    pub async fn device(&mut self, name: &str) -> Option<String> {
        let mut device_id = self.device_list.get(name).cloned();
        if device_id.is_none() {
            self.devices(false).await;
            device_id = self.device_list.get(name).cloned();
            if device_id.is_none() {
                return Some(format!("Device '{}' not found.", name));
            }
        }

        let device_id = device_id.unwrap();

        let res = self
            .api_manager
            .put(
                "api/spt-fwd/me/player",
                Some(json!({"device_ids": [device_id.to_string()]})),
                None,
            )
            .await;

        if res.is_ok() {
            let (status, _) = res.unwrap();
            if status.as_u16() == 200 || status.as_u16() == 204 {
                return Some(format!("Changing playback device to {}", name));
            } else if status.as_u16() == 401 || status.as_u16() == 403 {
                return Some("User not authorized".to_string());
            } else if status.as_u16() == 429 {
                return Some("Rate limit exceeded".to_string());
            } else {
                return Some(format!("Error: {}", status));
            }
        } else if res.is_err() {
            return Some(format!("Error: {}", res.unwrap_err()));
        }

        return Some("Unknown error.".to_string());
    }

    pub async fn devices(&mut self, human_readable: bool) -> Option<String> {
        let res = self
            .api_manager
            .get("api/spt-fwd/me/player/devices", None)
            .await;

        if res.is_ok() {
            let (status, json) = res.unwrap();
            if status.as_u16() == 200 {
                self.device_list.clear(); // Clear previous device list

                let mut devices_list = String::from("Available Devices:\n");
                if let Some(devices) = json["devices"].as_array() {
                    for device in devices {
                        if let (Some(name), Some(id)) =
                            (device["name"].as_str(), device["id"].as_str())
                        {
                            if human_readable {
                                devices_list.push_str(&format!("\t{} ({})\n", name, id));
                            } else {
                                devices_list.push_str(&format!("{}\n", id));
                            }
                            self.device_list.insert(name.to_string(), id.to_string());
                        }
                    }
                }
                return Some(devices_list);
            } else if status.as_u16() == 204 {
                return Some("No devices found.".to_string());
            } else if status.as_u16() == 401 || status.as_u16() == 403 {
                return Some("User not authorized".to_string());
            } else if status.as_u16() == 429 {
                return Some("Rate limit exceeded".to_string());
            }
        } else if res.is_err() {
            return Some(format!("Error: {}", res.unwrap_err()));
        }

        return Some("Unknown error.".to_string());
    }

    pub async fn queue(&mut self, human_readable: bool) -> Option<String> {
        let res = self
            .api_manager
            .get("api/spt-fwd/me/player/queue", None)
            .await;

        if res.is_ok() {
            let (status, json) = res.unwrap();
            if status.as_u16() == 200 {
                let mut q_list = String::new();

                if human_readable {
                    q_list.push_str("Now Playing:\n");
                    if !json["currently_playing"].is_null() {
                        q_list.push_str(&format!(
                            "\t{} - {} by {}\n",
                            json["currently_playing"]["name"],
                            json["currently_playing"]["album"]["name"],
                            json["currently_playing"]["artists"][0]["name"]
                        ));
                    } else {
                        q_list.push_str("\tNone\n");
                    }
                    q_list.push_str("Queue:\n");
                    for item in json["queue"].as_array().unwrap_or(&vec![]) {
                        q_list.push_str(&format!(
                            "\t{} - {} by {}\n",
                            item["name"], item["album"]["name"], item["artists"][0]["name"]
                        ));
                    }
                    if json["queue"].as_array().unwrap_or(&vec![]).len() == 0 {
                        q_list.push_str("\tNone\n");
                    }
                } else {
                    if !json["currently_playing"].is_null() {
                        q_list.push_str(&format!("{}\n", json["currently_playing"]["uri"]));
                    }
                    for item in json["queue"].as_array().unwrap_or(&vec![]) {
                        q_list.push_str(&format!("{}\n", item["uri"]));
                    }
                }

                return Some(q_list);
            } else if status.as_u16() == 204 {
                return Some("No queue found.".to_string());
            } else if status.as_u16() == 401 || status.as_u16() == 403 {
                return Some("User not authorized".to_string());
            } else if status.as_u16() == 429 {
                return Some("Rate limit exceeded".to_string());
            }
        } else if res.is_err() {
            return Some(format!("Error: {}", res.unwrap_err()));
        }

        return Some("Unknown error.".to_string());
    }

    pub async fn queue_add(&mut self, uris: Vec<String>) -> Option<String> {
        let n = uris.len();
        let mut results = Vec::with_capacity(n);

        for _ in 0..n {
            let params = HashMap::from([("uris".to_string(), uris[n].clone())]);
            let res = self
                .api_manager
                .post("api/spt-fwd/me/player/queue", None, Some(params))
                .await;
            results.push(res);
        }

        let mut output = Vec::new();

        let all_failed = results
            .iter()
            .filter(|res| match res {
                Ok((status, _)) => match status.as_u16() {
                    200 => false,
                    204 => false,
                    401 | 403 => {
                        output.push("User not authorized".to_string());
                        true
                    }
                    429 => {
                        output.push("Rate limit exceeded".to_string());
                        true
                    }
                    _ => {
                        output.push(format!("Error: {}", status));
                        true
                    }
                },
                Err(err) => {
                    output.push(format!("Error: {}", err));
                    true
                }
            })
            .count();

        output.insert(
            0,
            format!(
                "Added {} track{} to queue.",
                n - all_failed,
                if n - all_failed > 1 { "s" } else { "" }
            ),
        );

        return Some(output.join("\n"));
    }

    pub async fn recent(&mut self, n: u8, human_readable: bool) -> Option<String> {
        let params = HashMap::from([("limit".to_string(), n.to_string())]);

        let res = self
            .api_manager
            .get("api/spt-fwd/me/player/recently-played", Some(params))
            .await;

        if res.is_ok() {
            let (status, json) = res.unwrap();
            if status.as_u16() == 200 {
                let mut q_list = String::new();

                if human_readable {
                    q_list.push_str("Recently Played:\n");
                    for item in json["items"].as_array().unwrap_or(&vec![]) {
                        q_list.push_str(&format!(
                            "\t{} - {} by {}\n",
                            item["name"], item["album"]["name"], item["artists"][0]["name"]
                        ));
                    }
                    if json["items"].as_array().unwrap_or(&vec![]).len() == 0 {
                        q_list.push_str("\tNone\n");
                    }
                } else {
                    for item in json["queue"].as_array().unwrap_or(&vec![]) {
                        q_list.push_str(&format!("{}\n", item["uri"]));
                    }
                }

                return Some(q_list);
            } else if status.as_u16() == 204 {
                return Some("No queue found.".to_string());
            } else if status.as_u16() == 401 || status.as_u16() == 403 {
                return Some("User not authorized".to_string());
            } else if status.as_u16() == 429 {
                return Some("Rate limit exceeded".to_string());
            }
        } else if res.is_err() {
            return Some(format!("Error: {}", res.unwrap_err()));
        }

        return Some("Unknown error.".to_string());
    }
}
