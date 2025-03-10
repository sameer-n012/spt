use crate::core::api_manager::ApiManager;

#[derive(Debug)]
pub struct TransactionManager {
    api_manager: &'static ApiManager,
}

impl TransactionManager {
    pub fn new() -> Self {
        TransactionManager {
            api_manager: ApiManager::get_instance(),
        }
    }
}
