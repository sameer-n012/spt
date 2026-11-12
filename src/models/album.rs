use crate::util::errors::DataParseError;
use sqlx::query::Query;

#[derive(Debug, Clone)]
pub struct Album {
    pub uri: String,
    pub name: String,
    pub album_type: String,
    pub release_date: String,
}

impl PartialEq for Album {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri
    }
}

impl Eq for Album {}

impl Album {
    pub fn from_db_record(album_record: serde_json::Value) -> Result<Album, DataParseError> {
        Ok(Album {
            uri: album_record["uri"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            name: album_record["name"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            album_type: album_record["type"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            release_date: album_record["release_date"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
        })
    }

    pub fn from_api_record(album_record: serde_json::Value) -> Result<Album, DataParseError> {
        // Same as from_db_record, but for API input
        Self::from_db_record(album_record)
    }

    pub fn to_api_record(&self) -> serde_json::Value {
        serde_json::json!({
            "uri": self.uri,
            "name": self.name,
            "type": self.album_type,
            "release_date": self.release_date,
        })
    }

    pub fn generate_insert_query(
        &self,
    ) -> Query<'_, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'_>> {
        let query_str = "INSERT INTO albums (uri, name, type, release_date) VALUES (?, ?, ?, ?) \
            ON CONFLICT(uri) DO UPDATE SET name = excluded.name, type = excluded.type, \
            release_date = excluded.release_date";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.uri);
        query = query.bind(&self.name);
        query = query.bind(&self.album_type);
        query = query.bind(&self.release_date);
        query
    }

    pub fn generate_update_query(
        &self,
    ) -> Query<'_, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'_>> {
        let query_str = "UPDATE albums SET name = ?, type = ?, release_date = ? WHERE uri = ?";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.name);
        query = query.bind(&self.album_type);
        query = query.bind(&self.release_date);
        query = query.bind(&self.uri);
        query
    }

    pub fn generate_delete_query(
        &self,
    ) -> Query<'_, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'_>> {
        let query_str = "DELETE FROM albums WHERE uri = ?";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.uri);
        query
    }
}
