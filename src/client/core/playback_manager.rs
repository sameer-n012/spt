use crate::client::local_api_manager::ApiProxy;
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

    pub async fn now(&self) -> String {
        let res = self
            .api_manager
            .get("api/spt-fwd/me/player/currently-playing", None)
            .await;

        if res.is_ok() {
            let (status, json) = res.unwrap();
            println!("Status: {:?}", status);
            if status.as_u16() == 200 {
                return format!(
                    "Now playing: {} - {} by {}",
                    json["item"]["name"],
                    json["item"]["album"]["name"],
                    json["item"]["artists"][0]["name"],
                );
            } else if status.as_u16() == 204 {
                return "No track currently playing.".to_string();
            } else if status.as_u16() == 401 || status.as_u16() == 403 {
                return "User not authorized".to_string();
            } else if status.as_u16() == 429 {
                return "Rate limit exceeded".to_string();
            }
        } else if res.is_err() {
            return format!("Error: {}", res.unwrap_err());
        }

        return "Unknown error.".to_string();
    }

    pub async fn play(&self) {
        println!("Playing...");
    }

    pub async fn pause(&self) {
        println!("Pausing...");
    }

    pub async fn next(&self, n: u8) {
        println!("Skipping to next track...");
    }

    pub async fn previous(&self, n: u8) {
        println!("Skipping to previous track...");
    }

    pub async fn set_volume(&self, level: u8) {
        println!("Setting volume to {}%", level);
    }

    pub async fn get_volume(&self) -> String {
        let res = self
            .api_manager
            .get("api/spt-fwd/me/player/devices", None)
            .await;

        if res.is_ok() {
            let (status, json) = res.unwrap();
            if status.as_u16() == 200 {
                return format!(
                    "Volume: {}% ({})",
                    json["device"]["volume_percent"].to_string(),
                    json["device"]["name"],
                );
            } else if status.as_u16() == 204 {
                return "No devices found.".to_string();
            } else if status.as_u16() == 401 || status.as_u16() == 403 {
                return "User not authorized".to_string();
            } else if status.as_u16() == 429 {
                return "Rate limit exceeded".to_string();
            }
        } else if res.is_err() {
            return format!("Error: {}", res.unwrap_err());
        }

        return "Unknown error.".to_string();
    }

    pub async fn device(&mut self, name: &str) {
        println!("Changing playback device to {}", name);
        self.curr_device_name = Some(name.to_string());
    }

    pub async fn devices(&mut self, human_readable: bool) -> String {
        let res = self
            .api_manager
            .get("api/spt-fwd/me/player/devices", None)
            .await;

        if res.is_ok() {
            let (status, json) = res.unwrap();
            if status.as_u16() == 200 {
                println!("{}", json);

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
                return devices_list;
            } else if status.as_u16() == 204 {
                return "No devices found.".to_string();
            } else if status.as_u16() == 401 || status.as_u16() == 403 {
                return "User not authorized".to_string();
            } else if status.as_u16() == 429 {
                return "Rate limit exceeded".to_string();
            }
        } else if res.is_err() {
            return format!("Error: {}", res.unwrap_err());
        }

        return "Unknown error.".to_string();
    }
}
