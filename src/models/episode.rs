use serde_json::Value;
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
        let mut e = Episode {
            uri: episode_record["uri"]
                .as_str()
                .ok_or(DataParseError::new("uri"))?
                .to_string(),
            name: episode_record["name"]
                .as_str()
                .ok_or(DataParseError::new("name"))?
                .to_string(),
            show_uri: episode_record["show_uri"]
                .as_str()
                .ok_or(DataParseError::new("show_uri"))?
                .to_string(),
            episode_number: episode_record["episode_number"]
                .as_i64()
                .ok_or(DataParseError::new("episode_number"))? as i32,
            release_date: episode_record["release_date"]
                .as_str()
                .ok_or(DataParseError::new("release_date"))?
                .to_string(),
            description: episode_record["description"]
                .as_str()
                .map(|s| s.to_string()),
            duration_ms: episode_record["duration_ms"]
                .as_i64()
                .ok_or(DataParseError::new("duration_ms"))? as i32,
            explicit: episode_record["explicit"]
                .as_bool()
                .ok_or(DataParseError::new("explicit"))?,
            user_analytics: None,
        };

        e.user_analytics = if let Some(eua_record) = episode_user_analytics_record {
            Some(EpisodeUserAnalytics {
                user_id: eua_record["user_id"]
                    .as_str()
                    .ok_or(DataParseError::new("user_id"))?
                    .to_string(),
                play_count: eua_record["play_count"]
                    .as_i64()
                    .ok_or(DataParseError::new("play_count"))? as i32,
                skip_count: eua_record["skip_count"]
                    .as_i64()
                    .ok_or(DataParseError::new("skip_count"))? as i32,
                replay_count: eua_record["replay_count"]
                    .as_i64()
                    .ok_or(DataParseError::new("replay_count"))?
                    as i32,
                last_played: eua_record["last_played"].as_str().map(|s| s.to_string()),
            })
        } else {
            None
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
                .ok_or(DataParseError::new("uri"))?
                .to_string(),
            name: episode_record["name"]
                .as_str()
                .ok_or(DataParseError::new("name"))?
                .to_string(),
            show_uri: episode_record["show_uri"]
                .as_str()
                .ok_or(DataParseError::new("show_uri"))?
                .to_string(),
            episode_number: episode_record["episode_number"]
                .as_i64()
                .ok_or(DataParseError::new("episode_number"))? as i32,
            release_date: episode_record["release_date"]
                .as_str()
                .ok_or(DataParseError::new("release_date"))?
                .to_string(),
            description: episode_record["description"]
                .as_str()
                .map(|s| s.to_string()),
            duration_ms: episode_record["duration_ms"]
                .as_i64()
                .ok_or(DataParseError::new("duration_ms"))? as i32,
            explicit: episode_record["explicit"]
                .as_bool()
                .ok_or(DataParseError::new("explicit"))?,
            user_analytics: None,
        };

        e.user_analytics = if let Some(eua_record) = episode_user_analytics_record {
            Some(EpisodeUserAnalytics {
                user_id: eua_record["user_id"]
                    .as_str()
                    .ok_or(DataParseError::new("user_id"))?
                    .to_string(),
                play_count: eua_record["play_count"]
                    .as_i64()
                    .ok_or(DataParseError::new("play_count"))? as i32,
                skip_count: eua_record["skip_count"]
                    .as_i64()
                    .ok_or(DataParseError::new("skip_count"))? as i32,
                replay_count: eua_record["replay_count"]
                    .as_i64()
                    .ok_or(DataParseError::new("replay_count"))?
                    as i32,
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

    pub fn generate_insert_query(&self) -> Query {
        let query_str = "INSERT INTO episodes (uri, name, show_uri, episode_number, release_date, description, \
            duration_ms, explicit) VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
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

    pub fn generate_update_query(&self) -> Query {
        let query_str =
            "UPDATE episodes SET name = ?, show_uri = ?, episode_number = ?, release_date = ?, \
            description = ?, duration_ms = ?, explicit = ? WHERE uri = ?";
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

    pub fn generate_delete_query(&self) -> Query {
        let query_str = "DELETE FROM episodes WHERE uri = ?";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.uri);
        query
    }

    pub fn generate_user_analytics_insert_query(&self, user_id: &str) -> Option<Query> {
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

    pub fn generate_user_analytics_update_query(&self, user_id: &str) -> Option<Query> {
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

    pub fn generate_user_analytics_delete_query(&self) -> Option<Query> {
        let query_str = "DELETE FROM episode_user_analytics WHERE episode_uri = ?";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.uri);
        Some(query)
    }
}
