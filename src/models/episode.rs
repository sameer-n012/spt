use crate::util::errors::DataParseError;
use sqlx::query::Query;

#[derive(Debug, Clone)]
pub struct EpisodeUserAnalytics {
    pub user_id: String,
    pub play_count: i32,
    pub skip_count: i32,
    pub replay_count: i32,
    pub last_played: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Episode {
    pub uri: String,
    pub name: String,
    pub show_uri: String,
    pub episode_number: i32,
    pub release_date: String,
    pub description: Option<String>,
    pub duration_ms: i32,
    pub explicit: bool,
    pub user_analytics: Option<EpisodeUserAnalytics>,
}

impl PartialEq for Episode {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri
    }
}

impl Eq for Episode {}

impl Episode {
    pub fn from_db_record(
        episode_record: serde_json::Value,
        episode_user_analytics_record: Option<serde_json::Value>,
    ) -> Result<Episode, DataParseError> {
        let e = Episode {
            uri: episode_record["uri"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            name: episode_record["name"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            show_uri: episode_record["show_uri"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            episode_number: episode_record["episode_number"]
                .as_i64()
                .ok_or(DataParseError::FieldMissingError)? as i32,
            release_date: episode_record["release_date"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            description: episode_record["description"]
                .as_str()
                .map(|s| s.to_string()),
            duration_ms: episode_record["duration_ms"]
                .as_i64()
                .ok_or(DataParseError::FieldMissingError)? as i32,
            explicit: episode_record["explicit"]
                .as_bool()
                .ok_or(DataParseError::FieldMissingError)?,
            user_analytics: episode_user_analytics_record.map(|_| EpisodeUserAnalytics {
                user_id: episode_record["user_id"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                play_count: episode_record["play_count"].as_i64().unwrap_or(0) as i32,
                skip_count: episode_record["skip_count"].as_i64().unwrap_or(0) as i32,
                replay_count: episode_record["replay_count"].as_i64().unwrap_or(0) as i32,
                last_played: episode_record["last_played"]
                    .as_str()
                    .map(|s| s.to_string()),
            }),
        };

        Ok(e)
    }

    pub fn from_api_record(
        episode_record: serde_json::Value,
        episode_user_analytics_record: Option<serde_json::Value>,
    ) -> Result<Episode, DataParseError> {
        let mut e = Episode {
            uri: episode_record["uri"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            name: episode_record["name"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            show_uri: episode_record["show_uri"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            episode_number: episode_record["episode_number"]
                .as_i64()
                .ok_or(DataParseError::FieldMissingError)? as i32,
            release_date: episode_record["release_date"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            description: episode_record["description"]
                .as_str()
                .map(|s| s.to_string()),
            duration_ms: episode_record["duration_ms"]
                .as_i64()
                .ok_or(DataParseError::FieldMissingError)? as i32,
            explicit: episode_record["explicit"]
                .as_bool()
                .ok_or(DataParseError::FieldMissingError)?,
            user_analytics: None,
        };

        e.user_analytics = if let Some(eua_record) = episode_user_analytics_record {
            Some(EpisodeUserAnalytics {
                user_id: eua_record["user_id"]
                    .as_str()
                    .ok_or(DataParseError::FieldMissingError)?
                    .to_string(),
                play_count: eua_record["play_count"]
                    .as_i64()
                    .ok_or(DataParseError::FieldMissingError)? as i32,
                skip_count: eua_record["skip_count"]
                    .as_i64()
                    .ok_or(DataParseError::FieldMissingError)? as i32,
                replay_count: eua_record["replay_count"]
                    .as_i64()
                    .ok_or(DataParseError::FieldMissingError)? as i32,
                last_played: eua_record["last_played"].as_str().map(|s| s.to_string()),
            })
        } else {
            None
        };

        Ok(e)
    }

    pub fn to_api_record(&self) -> serde_json::Value {
        let user_analytics = if let Some(ua) = &self.user_analytics {
            serde_json::json!({
                "user_id": ua.user_id,
                "play_count": ua.play_count,
                "skip_count": ua.skip_count,
                "replay_count": ua.replay_count,
                "last_played": ua.last_played,
            })
        } else {
            serde_json::Value::Null
        };

        serde_json::json!({
            "uri": self.uri,
            "name": self.name,
            "show_uri": self.show_uri,
            "episode_number": self.episode_number,
            "release_date": self.release_date,
            "description": self.description,
            "duration_ms": self.duration_ms,
            "explicit": self.explicit,
            "user_analytics": user_analytics,
        })
    }

    pub fn generate_insert_query(
        &self,
    ) -> Query<'_, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'_>> {
        let query_str = "INSERT INTO episodes (uri, name, show_uri, episode_number, release_date, description, duration_ms, explicit) VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
            ON CONFLICT(uri) DO UPDATE SET name = excluded.name, show_uri = excluded.show_uri, \
            release_date = excluded.release_date, description = excluded.description, \
            duration_ms = excluded.duration_ms, explicit = excluded.explicit";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.uri);
        query = query.bind(&self.name);
        query = query.bind(&self.show_uri);
        query = query.bind(self.episode_number);
        query = query.bind(&self.release_date);
        query = query.bind(&self.description);
        query = query.bind(self.duration_ms);
        query = query.bind(self.explicit);
        query
    }

    pub fn generate_update_query<'q>(
        &'q self,
    ) -> Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
        let query_str = "UPDATE episodes SET name = ?, show_uri = ?, episode_number = ?, release_date = ?, description = ?, duration_ms = ?, explicit = ? WHERE uri = ?";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.name);
        query = query.bind(&self.show_uri);
        query = query.bind(self.episode_number);
        query = query.bind(&self.release_date);
        query = query.bind(&self.description);
        query = query.bind(self.duration_ms);
        query = query.bind(self.explicit);
        query = query.bind(&self.uri);
        query
    }

    pub fn generate_delete_query(
        &self,
    ) -> Query<'_, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'_>> {
        let query_str = "DELETE FROM episodes WHERE uri = ?";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.uri);
        query
    }

    pub fn generate_user_analytics_insert_query<'q>(
        &'q self,
        user_id: &'q str,
    ) -> Option<Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>> {
        if let Some(ua) = &self.user_analytics {
            let query_str = "INSERT INTO episode_user_analytics (user_id, episode_uri, play_count, skip_count, \
                replay_count, last_played) VALUES (?, ?, ?, ?, ?, ?)";
            let mut query = sqlx::query(query_str);
            query = query.bind(user_id);
            query = query.bind(&self.uri);
            query = query.bind(ua.play_count);
            query = query.bind(ua.skip_count);
            query = query.bind(ua.replay_count);
            query = query.bind(&ua.last_played);
            Some(query)
        } else {
            None
        }
    }

    pub fn generate_user_analytics_update_query<'q>(
        &'q self,
        user_id: &'q str,
    ) -> Option<Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>> {
        if let Some(ua) = &self.user_analytics {
            let query_str = "UPDATE episode_user_analytics SET play_count = ?, skip_count = ?, replay_count = ?, \
                last_played = ? WHERE user_id = ? AND episode_uri = ?";
            let mut query = sqlx::query(query_str);
            query = query.bind(ua.play_count);
            query = query.bind(ua.skip_count);
            query = query.bind(ua.replay_count);
            query = query.bind(&ua.last_played);
            query = query.bind(user_id);
            query = query.bind(&self.uri);
            Some(query)
        } else {
            None
        }
    }

    pub fn generate_user_analytics_delete_query<'q>(
        &'q self,
    ) -> Option<Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>> {
        let query_str = "DELETE FROM episode_user_analytics WHERE episode_uri = ?";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.uri);
        Some(query)
    }
}
