use crate::util::errors::DataParseError;
use sqlx::query::Query;

#[derive(Debug, Clone)]
pub struct Artist {
    pub uri: String,
    pub name: String,
}

impl PartialEq for Artist {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri
    }
}

impl Eq for Artist {}

impl Artist {
    pub fn from_db_record(artist_record: serde_json::Value) -> Result<Artist, DataParseError> {
        Ok(Artist {
            uri: artist_record["uri"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            name: artist_record["name"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
        })
    }

    pub fn from_api_record(artist_record: serde_json::Value) -> Result<Artist, DataParseError> {
        // identical to from_db_record for this struct
        Self::from_db_record(artist_record)
    }

    pub fn to_api_record(&self) -> serde_json::Value {
        serde_json::json!({
            "uri": self.uri,
            "name": self.name,
        })
    }

    pub fn generate_insert_query(
        &self,
    ) -> Query<'_, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'_>> {
        let query_str = "INSERT INTO artists (uri, name) VALUES (?, ?) \
            ON CONFLICT(uri) DO UPDATE SET name = excluded.name";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.uri);
        query = query.bind(&self.name);
        query
    }

    pub fn generate_update_query(
        &self,
    ) -> Query<'_, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'_>> {
        let query_str = "UPDATE artists SET name = ? WHERE uri = ?";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.name);
        query = query.bind(&self.uri);
        query
    }

    pub fn generate_delete_query(
        &self,
    ) -> Query<'_, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'_>> {
        let query_str = "DELETE FROM artists WHERE uri = ?";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.uri);
        query
    }
}
