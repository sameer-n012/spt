use crate::core::api_manager::ApiManager;

#[derive(Debug)]
pub struct SearchManager {
    api_manager: &'static ApiManager,
}

impl SearchManager {
    pub fn new() -> Self {
        SearchManager {
            api_manager: ApiManager::get_instance(),
        }
    }
}
