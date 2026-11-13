use crate::server::db::db_manager::DBManager;
use std::sync::Arc;

// stub: local changes queued for remote push (README "Remote Push / Transaction Management")
#[derive(Debug)]
pub struct TransactionManager {
    #[allow(dead_code)] // used once the feature is built
    db_manager: Arc<DBManager>,
}

impl TransactionManager {
    pub fn new(db_manager: Arc<DBManager>) -> Self {
        return TransactionManager { db_manager };
    }
}
