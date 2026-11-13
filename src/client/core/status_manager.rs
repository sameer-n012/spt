use crate::client::local_api_proxy::ApiProxy;

// stub: user/status info (planned `user` command)
#[derive(Debug)]
pub struct StatusManager<'a> {
    #[allow(dead_code)] // used once the feature is built
    api_manager: &'a mut ApiProxy,
}

impl<'a> StatusManager<'a> {
    pub fn new(api_manager: &'a mut ApiProxy) -> Self {
        return StatusManager { api_manager };
    }
}
