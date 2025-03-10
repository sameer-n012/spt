use crate::core::api_manager::ApiManager;

#[derive(Debug)]
pub struct PlaylistManager {
    api_manager: &'static mut ApiManager,
}

impl PlaylistManager {
    pub fn new() -> Self {
        PlaylistManager {
            api_manager: ApiManager::get_instance(),
        }
    }

    pub fn playlists(&self) {
        // self.api_manager.get("me/playlists", None);
    }
}
