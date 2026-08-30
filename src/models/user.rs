use crate::util::errors::DataParseError;
use sqlx::query::Query;

#[derive(Debug, Clone)]
pub struct User {
    pub user_id: String,
    pub display_name: String,
    pub refresh_token: Option<String>,
    pub refresh_token_expiry: Option<String>,
    pub password_hash: Option<String>,
    pub salt: Option<String>,
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.user_id == other.user_id
    }
}

impl Eq for User {}

impl User {
    pub fn from_db_record(user_record: serde_json::Value) -> Result<User, DataParseError> {
        Ok(User {
            user_id: user_record["user_id"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            display_name: user_record["display_name"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            refresh_token: user_record["refresh_token"].as_str().map(|s| s.to_string()),
            refresh_token_expiry: user_record["refresh_token_expiry"]
                .as_str()
                .map(|s| s.to_string()),
            password_hash: user_record["password_hash"].as_str().map(|s| s.to_string()),
            salt: user_record["salt"].as_str().map(|s| s.to_string()),
        })
    }

    pub fn from_api_record(user_record: serde_json::Value) -> Result<User, DataParseError> {
        // Same as from_db_record for this struct
        Self::from_db_record(user_record)
    }

    pub fn to_api_record(&self) -> serde_json::Value {
        serde_json::json!({
            "user_id": self.user_id,
            "display_name": self.display_name,
            "refresh_token": self.refresh_token,
            "refresh_token_expiry": self.refresh_token_expiry,
            "password_hash": self.password_hash,
            "salt": self.salt,
        })
    }

    pub fn generate_insert_query(
        &self,
    ) -> Query<'_, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'_>> {
        let query_str = "INSERT INTO users (user_id, display_name, refresh_token, refresh_token_expiry, password_hash, salt) VALUES (?, ?, ?, ?, ?, ?)";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.user_id);
        query = query.bind(&self.display_name);
        query = query.bind(&self.refresh_token);
        query = query.bind(&self.refresh_token_expiry);
        query = query.bind(&self.password_hash);
        query = query.bind(&self.salt);
        query
    }

    pub fn generate_update_query(
        &self,
    ) -> Query<'_, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'_>> {
        let query_str = "UPDATE users SET display_name = ?, refresh_token = ?, refresh_token_expiry = ?, password_hash = ?, salt = ? WHERE user_id = ?";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.display_name);
        query = query.bind(&self.refresh_token);
        query = query.bind(&self.refresh_token_expiry);
        query = query.bind(&self.password_hash);
        query = query.bind(&self.salt);
        query = query.bind(&self.user_id);
        query
    }

    pub fn generate_delete_query(
        &self,
    ) -> Query<'_, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'_>> {
        let query_str = "DELETE FROM users WHERE user_id = ?";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.user_id);
        query
    }
}
