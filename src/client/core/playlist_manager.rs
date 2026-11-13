use crate::client::local_api_proxy::ApiProxy;

// stub: playlist management (README "Playlist Management"), see CLAUDE.md playlist endpoints
#[derive(Debug)]
pub struct PlaylistManager<'a> {
    #[allow(dead_code)] // used once the feature is built
    api_manager: &'a mut ApiProxy,
}

impl<'a> PlaylistManager<'a> {
    pub fn new(api_manager: &'a mut ApiProxy) -> Self {
        return PlaylistManager { api_manager };
    }

    pub fn playlists(&self) {
        // self.api_manager.get("api/spt-fwd/me/playlists", None);
    }
}
