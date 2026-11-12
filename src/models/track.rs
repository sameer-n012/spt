use crate::util::errors::DataParseError;
use sqlx::query::Query;

#[derive(Debug, Clone)]
pub struct TrackAnalytics {
    pub popularity: Option<i32>,
    pub acousticness: Option<f32>,
    pub danceability: Option<f32>,
    pub energy: Option<f32>,
    pub instrumentalness: Option<f32>,
    pub liveness: Option<f32>,
    pub loudness: Option<f32>,
    pub speechiness: Option<f32>,
    pub tempo: Option<f32>,
    pub valence: Option<f32>,
    pub time_signature: Option<i32>,
    pub pitch_class_key: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct TrackUserAnalytics {
    pub user_id: String,
    pub play_count: i32,
    pub skip_count: i32,
    pub replay_count: i32,
    pub last_played: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub uri: String,
    pub name: String,
    pub album_uri: String,
    pub track_number: i32,
    pub disc_number: i32,
    pub duration_ms: i32,
    pub explicit: bool,
    pub analytics: Option<TrackAnalytics>,
    pub user_analytics: Option<TrackUserAnalytics>,
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        return self.uri == other.uri;
    }
}

impl Eq for Track {}

impl Track {
    pub fn from_db_record(
        track_record: serde_json::Value,
        track_analytics_record: Option<serde_json::Value>,
    ) -> Result<Track, DataParseError> {
        let mut t = Track {
            uri: track_record["uri"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            name: track_record["name"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            album_uri: track_record["album_uri"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            track_number: track_record["track_number"]
                .as_i64()
                .ok_or(DataParseError::FieldMissingError)? as i32,
            disc_number: track_record["disc_number"]
                .as_i64()
                .ok_or(DataParseError::FieldMissingError)? as i32,
            duration_ms: track_record["duration_ms"]
                .as_i64()
                .ok_or(DataParseError::FieldMissingError)? as i32,
            explicit: track_record["explicit"]
                .as_bool()
                .ok_or(DataParseError::FieldMissingError)?,
            analytics: None,
            user_analytics: None,
        };

        t.analytics = Some(TrackAnalytics {
            popularity: track_record["popularity"].as_i64().map(|v| v as i32),
            acousticness: track_record["acousticness"].as_f64().map(|v| v as f32),
            danceability: track_record["danceability"].as_f64().map(|v| v as f32),
            energy: track_record["energy"].as_f64().map(|v| v as f32),
            instrumentalness: track_record["instrumentalness"].as_f64().map(|v| v as f32),
            liveness: track_record["liveness"].as_f64().map(|v| v as f32),
            loudness: track_record["loudness"].as_f64().map(|v| v as f32),
            speechiness: track_record["speechiness"].as_f64().map(|v| v as f32),
            tempo: track_record["tempo"].as_f64().map(|v| v as f32),
            valence: track_record["valence"].as_f64().map(|v| v as f32),
            time_signature: track_record["time_signature"].as_i64().map(|v| v as i32),
            pitch_class_key: track_record["pitch_class_key"].as_i64().map(|v| v as i32),
        });

        t.user_analytics = if let Some(ta_record) = track_analytics_record {
            Some(TrackUserAnalytics {
                user_id: ta_record["user_id"]
                    .as_str()
                    .ok_or(DataParseError::FieldMissingError)?
                    .to_string(),
                play_count: ta_record["play_count"]
                    .as_i64()
                    .ok_or(DataParseError::FieldMissingError)? as i32,
                skip_count: ta_record["skip_count"]
                    .as_i64()
                    .ok_or(DataParseError::FieldMissingError)? as i32,
                replay_count: ta_record["replay_count"]
                    .as_i64()
                    .ok_or(DataParseError::FieldMissingError)? as i32,
                last_played: ta_record["last_played"].as_str().map(|s| s.to_string()),
            })
        } else {
            None
        };

        Ok(t)
    }

    pub fn from_api_record(
        track_record: serde_json::Value,
        track_analytics_record: Option<serde_json::Value>,
    ) -> Result<Track, DataParseError> {
        let mut t = Track {
            uri: track_record["uri"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            name: track_record["name"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            album_uri: track_record["album_uri"]
                .as_str()
                .ok_or(DataParseError::FieldMissingError)?
                .to_string(),
            track_number: track_record["track_number"]
                .as_i64()
                .ok_or(DataParseError::FieldMissingError)? as i32,
            disc_number: track_record["disc_number"]
                .as_i64()
                .ok_or(DataParseError::FieldMissingError)? as i32,
            duration_ms: track_record["duration_ms"]
                .as_i64()
                .ok_or(DataParseError::FieldMissingError)? as i32,
            explicit: track_record["explicit"]
                .as_bool()
                .ok_or(DataParseError::FieldMissingError)?,
            analytics: None,
            user_analytics: None,
        };

        t.analytics = Some(TrackAnalytics {
            popularity: track_record["popularity"].as_i64().map(|v| v as i32),
            acousticness: track_record["acousticness"].as_f64().map(|v| v as f32),
            danceability: track_record["danceability"].as_f64().map(|v| v as f32),
            energy: track_record["energy"].as_f64().map(|v| v as f32),
            instrumentalness: track_record["instrumentalness"].as_f64().map(|v| v as f32),
            liveness: track_record["liveness"].as_f64().map(|v| v as f32),
            loudness: track_record["loudness"].as_f64().map(|v| v as f32),
            speechiness: track_record["speechiness"].as_f64().map(|v| v as f32),
            tempo: track_record["tempo"].as_f64().map(|v| v as f32),
            valence: track_record["valence"].as_f64().map(|v| v as f32),
            time_signature: track_record["time_signature"].as_i64().map(|v| v as i32),
            pitch_class_key: track_record["pitch_class_key"].as_i64().map(|v| v as i32),
        });

        t.user_analytics = if let Some(ta_record) = track_analytics_record {
            Some(TrackUserAnalytics {
                user_id: ta_record["user_id"]
                    .as_str()
                    .ok_or(DataParseError::FieldMissingError)?
                    .to_string(),
                play_count: ta_record["play_count"]
                    .as_i64()
                    .ok_or(DataParseError::FieldMissingError)? as i32,
                skip_count: ta_record["skip_count"]
                    .as_i64()
                    .ok_or(DataParseError::FieldMissingError)? as i32,
                replay_count: ta_record["replay_count"]
                    .as_i64()
                    .ok_or(DataParseError::FieldMissingError)? as i32,
                last_played: ta_record["last_played"].as_str().map(|s| s.to_string()),
            })
        } else {
            None
        };

        Ok(t)
    }

    pub fn to_api_record(&self) -> serde_json::Value {
        let analytics = if let Some(ta) = &self.analytics {
            serde_json::json!({
                "popularity": ta.popularity,
                "acousticness": ta.acousticness,
                "danceability": ta.danceability,
                "energy": ta.energy,
                "instrumentalness": ta.instrumentalness,
                "liveness": ta.liveness,
                "loudness": ta.loudness,
                "speechiness": ta.speechiness,
                "tempo": ta.tempo,
                "valence": ta.valence,
                "time_signature": ta.time_signature,
                "pitch_class_key": ta.pitch_class_key,
            })
        } else {
            serde_json::Value::Null
        };

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
            "album_uri": self.album_uri,
            "track_number": self.track_number,
            "disc_number": self.disc_number,
            "duration_ms": self.duration_ms,
            "explicit": self.explicit,
            "analytics": analytics,
            "user_analytics": user_analytics,
        })
    }

    pub fn generate_insert_query<'q>(
        &'q self,
    ) -> Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
        let query_str = "INSERT INTO tracks (uri, name, album_uri, track_number, disc_number, \
            duration_ms, explicit, popularity, acousticness, danceability, energy, instrumentalness, \
            liveness, loudness, speechiness, tempo, valence, time_signature, pitch_class_key) \
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
            ON CONFLICT(uri) DO UPDATE SET name = excluded.name, album_uri = excluded.album_uri, \
            track_number = excluded.track_number, disc_number = excluded.disc_number, \
            duration_ms = excluded.duration_ms, explicit = excluded.explicit";
        // re-caching only updates metadata, audio features stay as they are
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.uri);
        query = query.bind(&self.name);
        query = query.bind(&self.album_uri);
        query = query.bind(self.track_number);
        query = query.bind(self.disc_number);
        query = query.bind(self.duration_ms);
        query = query.bind(self.explicit);
        if let Some(analytics) = &self.analytics {
            query = query.bind(analytics.popularity.or(None::<i32>));
            query = query.bind(analytics.acousticness.or(None::<f32>));
            query = query.bind(analytics.danceability.or(None::<f32>));
            query = query.bind(analytics.energy.or(None::<f32>));
            query = query.bind(analytics.instrumentalness.or(None::<f32>));
            query = query.bind(analytics.liveness.or(None::<f32>));
            query = query.bind(analytics.loudness.or(None::<f32>));
            query = query.bind(analytics.speechiness.or(None::<f32>));
            query = query.bind(analytics.tempo.or(None::<f32>));
            query = query.bind(analytics.valence.or(None::<f32>));
            query = query.bind(analytics.time_signature.or(None::<i32>));
            query = query.bind(analytics.pitch_class_key.or(None::<i32>));
        } else {
            for _ in 0..12 {
                query = query.bind(None::<i32>);
            }
        }

        return query;
    }

    pub fn generate_update_query<'q>(
        &'q self,
    ) -> Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
        let query_str = "UPDATE tracks SET name = ?, album_uri = ?, track_number = ?, disc_number = ?, \
            duration_ms = ?, explicit = ?, popularity = ?, acousticness = ?, danceability = ?, energy = ?, \
            instrumentalness = ?, liveness = ?, loudness = ?, speechiness = ?, tempo = ?, valence = ?, \
            time_signature = ?, pitch_class_key = ? WHERE uri = ?";

        let mut query = sqlx::query(query_str);
        query = query.bind(&self.name);
        query = query.bind(&self.album_uri);
        query = query.bind(self.track_number);
        query = query.bind(self.disc_number);
        query = query.bind(self.duration_ms);
        query = query.bind(self.explicit);
        if let Some(analytics) = &self.analytics {
            query = query.bind(analytics.popularity.or(None::<i32>));
            query = query.bind(analytics.acousticness.or(None::<f32>));
            query = query.bind(analytics.danceability.or(None::<f32>));
            query = query.bind(analytics.energy.or(None::<f32>));
            query = query.bind(analytics.instrumentalness.or(None::<f32>));
            query = query.bind(analytics.liveness.or(None::<f32>));
            query = query.bind(analytics.loudness.or(None::<f32>));
            query = query.bind(analytics.speechiness.or(None::<f32>));
            query = query.bind(analytics.tempo.or(None::<f32>));
            query = query.bind(analytics.valence.or(None::<f32>));
            query = query.bind(analytics.time_signature.or(None::<i32>));
            query = query.bind(analytics.pitch_class_key.or(None::<i32>));
        } else {
            for _ in 0..12 {
                query = query.bind(None::<i32>);
            }
        }
        query = query.bind(&self.uri);

        return query;
    }

    pub fn generate_delete_query<'q>(
        &'q self,
    ) -> Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
        let query_str = "DELETE FROM tracks WHERE uri = ?";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.uri);
        return query;
    }

    pub fn generate_user_analytics_insert_query<'q>(
        &'q self,
        user_id: &'q str,
    ) -> Option<Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>> {
        if let Some(ua) = &self.user_analytics {
            let query_str = "INSERT INTO track_user_analytics (user_id, track_uri, play_count, \
                skip_count, replay_count, last_played) VALUES (?, ?, ?, ?, ?, ?)";
            let mut query = sqlx::query(query_str);
            query = query.bind(user_id);
            query = query.bind(&self.uri);
            query = query.bind(ua.play_count);
            query = query.bind(ua.skip_count);
            query = query.bind(ua.replay_count);
            query = query.bind(&ua.last_played);

            return Some(query);
        }
        return None;
    }

    pub fn generate_user_analytics_update_query<'q>(
        &'q self,
        user_id: &'q str,
    ) -> Option<Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>> {
        if let Some(ua) = &self.user_analytics {
            let query_str = "UPDATE track_user_analytics SET play_count = ?, skip_count = ?, \
                replay_count = ?, last_played = ? WHERE user_id = ? AND track_uri = ?";
            let mut query = sqlx::query(query_str);
            query = query.bind(ua.play_count);
            query = query.bind(ua.skip_count);
            query = query.bind(ua.replay_count);
            query = query.bind(&ua.last_played);
            query = query.bind(user_id);
            query = query.bind(&self.uri);

            return Some(query);
        }
        return None;
    }

    pub fn generate_user_analytics_delete_query(
        &self,
    ) -> Query<'_, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'_>> {
        let query_str = "DELETE FROM track_user_analytics WHERE uri = ?";
        let mut query = sqlx::query(query_str);
        query = query.bind(&self.uri);
        return query;
    }
}
