use crate::client::local_api_manager::ApiProxy;

#[derive(Debug)]
pub struct PlaybackManager<'a> {
    curr_device_id: Option<String>,
    api_manager: &'a mut ApiProxy,
}

impl<'a> PlaybackManager<'a> {
    pub fn new(api_manager: &'a mut ApiProxy) -> Self {
        return PlaybackManager {
            curr_device_id: None,
            api_manager,
        };
    }

    pub async fn now(&mut self) -> String {
        // let res = self
        //     .api_manager
        //     .get("me/player/currently-playing", None)
        //     .await;
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
                    json["item"]["artists"][0]["name"],
                    json["item"]["album"]["name"]
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

    pub async fn get_volume(&self) {
        println!("Getting volume...");
    }

    pub async fn device(&mut self, name: &str) {
        println!("Changing playback device to {}", name);
        self.curr_device_id = Some(name.to_string());
    }
}
