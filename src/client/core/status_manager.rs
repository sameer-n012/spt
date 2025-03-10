use crate::core::api_manager::ApiManager;

#[derive(Debug)]
pub struct StatusManager<'a> {
    curr_device_id: Option<String>,
    api_manager: &'a mut ApiManager,
}

impl<'a> StatusManager<'a> {
    pub fn new(api_manager: &'a mut ApiManager) -> Self {
        return StatusManager {
            curr_device_id: None,
            api_manager,
        };
    }
}
