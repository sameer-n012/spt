use crate::util::errors::DataParseError;
use sqlx::query::Query;

#[derive(Debug, Clone)]
pub struct Playlist {
    pub uri: String,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub public: bool,
    pub collaborative: bool,
    pub snapshot_id: String,
}

impl PartialEq for Playlist {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri
    }
}

impl Eq for Playlist {}

impl Playlist {
    pub fn from_db_record(playlist_record: serde_json::Value) -> Result<Playlist, DataParseError> {
        Ok(Playlist {
            uri: playlist_record["uri"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            user_id: playlist_record["user_id"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            name: playlist_record["name"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            description: playlist_record["description"]
                .as_str()
                .map(|s| s.to_string()),
            public: playlist_record["public"]
                .as_bool()
                .ok_or(DataParseError::FieldMissingError)?,
            collaborative: playlist_record["collaborative"]
                .as_bool()
                .ok_or(DataParseError::FieldMissingError)?,
            snapshot_id: playlist_record["snapshot_id"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
        })
    }

    pub fn from_api_record(playlist_record: serde_json::Value) -> Result<Playlist, DataParseError> {
        // Same as from_db_record for now
        Self::from_db_record(playlist_record)
    }

    pub fn to_api_record(&self) -> serde_json::Value {
        serde_json::json!({
            "uri": self.uri,
            "user_id": self.user_id,
            "name": self.name,
            "description": self.description,
            "public": self.public,
            "collaborative": self.collaborative,
            "snapshot_id": self.snapshot_id,
        })
    }

    pub fn generate_insert_query(
        &self,
    ) -> Query<'_, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'_>> {
        let query_str = "INSERT INTO playlists (uri, user_id, name, description, public, collaborative, snapshot_id) VALUES (?, ?, ?, ?, ?, ?, ?)";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.uri);
        query = query.bind(&self.user_id);
        query = query.bind(&self.name);
        query = query.bind(&self.description);
        query = query.bind(&self.public);
        query = query.bind(&self.collaborative);
        query = query.bind(&self.snapshot_id);
        query
    }

    pub fn generate_update_query(
        &self,
    ) -> Query<'_, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'_>> {
        let query_str = "UPDATE playlists SET user_id = ?, name = ?, description = ?, public = ?, collaborative = ?, snapshot_id = ? WHERE uri = ?";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.user_id);
        query = query.bind(&self.name);
        query = query.bind(&self.description);
        query = query.bind(self.public);
        query = query.bind(self.collaborative);
        query = query.bind(&self.snapshot_id);
        query = query.bind(&self.uri);
        query
    }

    pub fn generate_delete_query(
        &self,
    ) -> Query<'_, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'_>> {
        let query_str = "DELETE FROM playlists WHERE uri = ?";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.uri);
        query
    }
}
