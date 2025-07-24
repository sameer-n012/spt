use crate::util::errors::DBError;
use std::env;
use std::fs;
use std::path::Path;
use std::str::FromStr;

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteConnection},
    SqlitePool,
};

#[derive(Debug, Clone)]
pub struct DBManager {
    connection: Option<SqlitePool>,
    db_url: String,
    db_port: u16,
}

impl DBManager {
    pub fn new(db_url: String, db_port: u16) -> Self {
        DBManager {
            connection: None,
            db_url,
            db_port,
        }
    }

    pub async fn setup(&mut self) -> Result<(), DBError> {
        let db_addr = if self.db_port > 0 {
            format!("{}:{}", self.db_url, self.db_port)
        } else {
            self.db_url.clone()
        };

        let db_exists = Path::new(&self.db_url).exists();

        let mut opts =
            SqliteConnectOptions::from_str(&db_addr).map_err(|_| DBError::ConnectionError)?;
        if !db_exists {
            opts = opts.create_if_missing(true);
        }

        let conn = SqlitePool::connect_with(opts)
            .await
            .map_err(|_| DBError::ConnectionError)?;

        self.connection = Some(conn);

        if !db_exists {
            let setup_script =
                fs::read_to_string("db_setup.sql").map_err(|_| DBError::SetupError)?;

            let _ = sqlx::query(&setup_script)
                .execute(self.connection.as_ref().unwrap())
                .await
                .map_err(|_| DBError::SetupError)?;
        }

        Ok(())
    }
}
