use crate::core::api_manager::ApiManager;

#[derive(Debug)]
pub struct QueueManager {
    api_manager: &'static ApiManager,
}

impl QueueManager {
    pub fn new() -> Self {
        QueueManager {
            api_manager: ApiManager::get_instance(),
        }
    }
}
