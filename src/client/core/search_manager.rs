use crate::client::local_api_proxy::ApiProxy;

// stub: search (README "Search"), dev mode caps search limit at 10
#[derive(Debug)]
pub struct SearchManager<'a> {
    #[allow(dead_code)] // used once the feature is built
    api_manager: &'a mut ApiProxy,
}

impl<'a> SearchManager<'a> {
    pub fn new(api_manager: &'a mut ApiProxy) -> Self {
        return SearchManager { api_manager };
    }
}
