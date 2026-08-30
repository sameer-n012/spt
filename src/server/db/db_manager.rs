use crate::util::errors::DBError;
use crate::util::uri_helper::{get_uri_type, UriType};
use sqlx::Column;
use sqlx::Row;
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

    // --- DB Caching: Get/Insert Methods for Each Item Type ---

    pub async fn get_track(&self, uri: &str) -> Result<Option<serde_json::Value>, DBError> {
        let res = sqlx::query("SELECT * FROM tracks WHERE uri = $1")
            .bind(uri)
            .fetch_optional(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?;
        Ok(res.map(|row| {
            let mut obj = serde_json::Map::new();
            for col in row.columns() {
                let val: Result<serde_json::Value, _> = row
                    .try_get(col.name())
                    .map(|v: String| serde_json::Value::String(v));
                if let Ok(v) = val {
                    obj.insert(col.name().to_string(), v);
                }
            }
            serde_json::Value::Object(obj)
        }))
    }

    pub async fn insert_track(&self, track: &crate::models::track::Track) -> Result<(), DBError> {
        let query = track.generate_insert_query();
        query
            .execute(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?;
        Ok(())
    }

    pub async fn get_album(&self, uri: &str) -> Result<Option<serde_json::Value>, DBError> {
        let res = sqlx::query("SELECT * FROM albums WHERE uri = $1")
            .bind(uri)
            .fetch_optional(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?;
        Ok(res.map(|row| {
            let mut obj = serde_json::Map::new();
            for col in row.columns() {
                let val: Result<serde_json::Value, _> = row
                    .try_get(col.name())
                    .map(|v: String| serde_json::Value::String(v));
                if let Ok(v) = val {
                    obj.insert(col.name().to_string(), v);
                }
            }
            serde_json::Value::Object(obj)
        }))
    }

    pub async fn insert_album(&self, album: &crate::models::album::Album) -> Result<(), DBError> {
        let query = album.generate_insert_query();
        query
            .execute(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?;
        Ok(())
    }

    pub async fn get_artist(&self, uri: &str) -> Result<Option<serde_json::Value>, DBError> {
        let res = sqlx::query("SELECT * FROM artists WHERE uri = $1")
            .bind(uri)
            .fetch_optional(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?;
        Ok(res.map(|row| {
            let mut obj = serde_json::Map::new();
            for col in row.columns() {
                let val: Result<serde_json::Value, _> = row
                    .try_get(col.name())
                    .map(|v: String| serde_json::Value::String(v));
                if let Ok(v) = val {
                    obj.insert(col.name().to_string(), v);
                }
            }
            serde_json::Value::Object(obj)
        }))
    }

    pub async fn insert_artist(
        &self,
        artist: &crate::models::artist::Artist,
    ) -> Result<(), DBError> {
        let query = artist.generate_insert_query();
        query
            .execute(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?;
        Ok(())
    }

    pub async fn get_playlist(&self, uri: &str) -> Result<Option<serde_json::Value>, DBError> {
        let res = sqlx::query("SELECT * FROM playlists WHERE uri = $1")
            .bind(uri)
            .fetch_optional(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?;
        Ok(res.map(|row| {
            let mut obj = serde_json::Map::new();
            for col in row.columns() {
                let val: Result<serde_json::Value, _> = row
                    .try_get(col.name())
                    .map(|v: String| serde_json::Value::String(v));
                if let Ok(v) = val {
                    obj.insert(col.name().to_string(), v);
                }
            }
            serde_json::Value::Object(obj)
        }))
    }

    pub async fn insert_playlist(
        &self,
        playlist: &crate::models::playlist::Playlist,
    ) -> Result<(), DBError> {
        let query = playlist.generate_insert_query();
        query
            .execute(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?;
        Ok(())
    }

    pub async fn get_episode(&self, uri: &str) -> Result<Option<serde_json::Value>, DBError> {
        let res = sqlx::query("SELECT * FROM episodes WHERE uri = $1")
            .bind(uri)
            .fetch_optional(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?;
        Ok(res.map(|row| {
            let mut obj = serde_json::Map::new();
            for col in row.columns() {
                let val: Result<serde_json::Value, _> = row
                    .try_get(col.name())
                    .map(|v: String| serde_json::Value::String(v));
                if let Ok(v) = val {
                    obj.insert(col.name().to_string(), v);
                }
            }
            serde_json::Value::Object(obj)
        }))
    }

    pub async fn insert_episode(
        &self,
        episode: &crate::models::episode::Episode,
    ) -> Result<(), DBError> {
        let query = episode.generate_insert_query();
        query
            .execute(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?;
        Ok(())
    }

    pub async fn get_show(&self, uri: &str) -> Result<Option<serde_json::Value>, DBError> {
        let res = sqlx::query("SELECT * FROM shows WHERE uri = $1")
            .bind(uri)
            .fetch_optional(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?;
        Ok(res.map(|row| {
            let mut obj = serde_json::Map::new();
            for col in row.columns() {
                let val: Result<serde_json::Value, _> = row
                    .try_get(col.name())
                    .map(|v: String| serde_json::Value::String(v));
                if let Ok(v) = val {
                    obj.insert(col.name().to_string(), v);
                }
            }
            serde_json::Value::Object(obj)
        }))
    }

    pub async fn insert_show(&self, show: &crate::models::show::Show) -> Result<(), DBError> {
        let query = show.generate_insert_query();
        query
            .execute(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?;
        Ok(())
    }

    // --- Cached OAuth token storage (dev/test convenience, see cached_auth_tokens) ---

    pub async fn save_cached_auth_tokens(
        &self,
        spotify_client_id: &str,
        access_token: Option<&str>,
        access_token_expiry_unix: Option<i64>,
        refresh_token: &str,
        updated_at_unix: i64,
    ) -> Result<(), DBError> {
        sqlx::query(
            "INSERT INTO cached_auth_tokens \
                (spotify_client_id, access_token, access_token_expiry, refresh_token, updated_at) \
                VALUES ($1, $2, $3, $4, $5) \
                ON CONFLICT(spotify_client_id) DO UPDATE SET \
                access_token = excluded.access_token, \
                access_token_expiry = excluded.access_token_expiry, \
                refresh_token = excluded.refresh_token, \
                updated_at = excluded.updated_at",
        )
        .bind(spotify_client_id)
        .bind(access_token)
        .bind(access_token_expiry_unix)
        .bind(refresh_token)
        .bind(updated_at_unix)
        .execute(self.connection.as_ref().unwrap())
        .await
        .map_err(|_| DBError::QueryError)?;

        Ok(())
    }

    pub async fn load_cached_auth_tokens(
        &self,
        spotify_client_id: &str,
    ) -> Result<Option<(Option<String>, Option<i64>, String)>, DBError> {
        let row = sqlx::query(
            "SELECT access_token, access_token_expiry, refresh_token \
                FROM cached_auth_tokens WHERE spotify_client_id = $1",
        )
        .bind(spotify_client_id)
        .fetch_optional(self.connection.as_ref().unwrap())
        .await
        .map_err(|_| DBError::QueryError)?;

        Ok(row.map(|r| {
            let access_token: Option<String> = r.try_get("access_token").unwrap_or(None);
            let access_token_expiry: Option<i64> = r.try_get("access_token_expiry").unwrap_or(None);
            let refresh_token: String = r.try_get("refresh_token").unwrap_or_default();
            (access_token, access_token_expiry, refresh_token)
        }))
    }

    // Removes a cached token entry once its refresh token is confirmed dead (e.g. Spotify's
    // token endpoint returned invalid_grant), so a future server start doesn't keep retrying a
    // refresh token that can never succeed again before falling back to browser auth.
    pub async fn delete_cached_auth_tokens(&self, spotify_client_id: &str) -> Result<(), DBError> {
        sqlx::query("DELETE FROM cached_auth_tokens WHERE spotify_client_id = $1")
            .bind(spotify_client_id)
            .execute(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?;

        Ok(())
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
            let setup_script = fs::read_to_string("scripts/sql/db_setup.sql")
                .map_err(|_| DBError::SetupError)?;

            let _ = sqlx::query(&setup_script)
                .execute(self.connection.as_ref().unwrap())
                .await
                .map_err(|_| DBError::SetupError)?;
        }

        Ok(())
    }

    pub fn increment_play_count(&self, user_id: String, uri: String) -> Result<(), DBError> {
        let t = get_uri_type(&uri);

        let conn = match &self.connection {
            Some(c) => c,
            None => return Err(DBError::ConnectionError),
        };

        match t {
            UriType::Track => {
                futures::executor::block_on(async {
                    sqlx::query(
                        "UPDATE user_track_analytics SET play_count = play_count + 1 \
                        WHERE user_id = $1 AND track_uri = $2",
                    )
                    .bind(&user_id)
                    .bind(&uri)
                    .execute(conn)
                    .await
                    .map_err(|_| DBError::QueryError)
                })?;
            }
            UriType::Episode => {
                futures::executor::block_on(async {
                    sqlx::query(
                        "UPDATE user_track_episode_analytics SET play_count = play_count + 1 \
                        WHERE user_id = $1 AND episode_uri = $2",
                    )
                    .bind(&user_id)
                    .bind(&uri)
                    .execute(conn)
                    .await
                    .map_err(|_| DBError::QueryError)
                })?;
            }
            _ => {
                return Err(DBError::InvalidURI);
            }
        }

        Ok(())
    }

    pub fn increment_skip_count(&self, user_id: String, uri: String) -> Result<(), DBError> {
        let t = get_uri_type(&uri);

        let conn = match &self.connection {
            Some(c) => c,
            None => return Err(DBError::ConnectionError),
        };

        match t {
            UriType::Track => {
                futures::executor::block_on(async {
                    sqlx::query(
                        "UPDATE user_track_analytics SET skip_count = skip_count + 1 \
                        WHERE user_id = $1 AND track_uri = $2",
                    )
                    .bind(&user_id)
                    .bind(&uri)
                    .execute(conn)
                    .await
                    .map_err(|_| DBError::QueryError)
                })?;
            }
            UriType::Episode => {
                futures::executor::block_on(async {
                    sqlx::query(
                        "UPDATE user_track_episode_analytics SET skip_count = skip_count + 1 \
                        WHERE user_id = $1 AND episode_uri = $2",
                    )
                    .bind(&user_id)
                    .bind(&uri)
                    .execute(conn)
                    .await
                    .map_err(|_| DBError::QueryError)
                })?;
            }
            _ => {
                return Err(DBError::InvalidURI);
            }
        }

        Ok(())
    }

    pub fn increment_replay_count(&self, user_id: String, uri: String) -> Result<(), DBError> {
        let t = get_uri_type(&uri);

        let conn = match &self.connection {
            Some(c) => c,
            None => return Err(DBError::ConnectionError),
        };

        match t {
            UriType::Track => {
                futures::executor::block_on(async {
                    sqlx::query(
                        "UPDATE user_track_analytics SET replay_count = replay_count + 1 \
                        WHERE user_id = $1 AND track_uri = $2",
                    )
                    .bind(&user_id)
                    .bind(&uri)
                    .execute(conn)
                    .await
                    .map_err(|_| DBError::QueryError)
                })?;
            }
            UriType::Episode => {
                futures::executor::block_on(async {
                    sqlx::query(
                        "UPDATE user_track_episode_analytics SET replay_count = replay_count + 1 \
                        WHERE user_id = $1 AND episode_uri = $2",
                    )
                    .bind(&user_id)
                    .bind(&uri)
                    .execute(conn)
                    .await
                    .map_err(|_| DBError::QueryError)
                })?;
            }
            _ => {
                return Err(DBError::InvalidURI);
            }
        }

        Ok(())
    }

    pub async fn check_item_in_db(&self, uri: &str) -> Result<bool, DBError> {
        let table_name = match get_uri_type(uri) {
            UriType::Track => "tracks",
            UriType::Episode => "episodes",
            UriType::Album => "albums",
            UriType::Artist => "artists",
            UriType::Playlist => "playlists",
            UriType::Show => "shows",
            _ => {
                return Err(DBError::InvalidURI);
            }
        };

        let res = sqlx::query(&format!(
            "SELECT COUNT(1) as count FROM {} WHERE uri = $1",
            table_name
        ))
        .bind(uri)
        .fetch_one(self.connection.as_ref().unwrap())
        .await
        .map_err(|_| DBError::QueryError)?;

        let count: i64 = res.try_get("count").unwrap_or(0);
        return Ok(count > 0);
    }

    pub async fn update_track_information(&self, uri: &str) -> Result<(), DBError> {
        // TODO implement
        return Ok(());
    }

    pub async fn update_item_information_if_needed(
        &self,
        uri: &str,
        force_update: bool,
    ) -> Result<(), DBError> {
        if force_update || !self.check_item_in_db(uri).await? {
            // get whether item is track or episode in match
            match get_uri_type(uri) {
                UriType::Track => {
                    // fetch track information from Spotify API and update DB
                }
                UriType::Episode => {
                    // fetch episode information from Spotify API and update DB
                }
                UriType::Album => {
                    // fetch album information from Spotify API and update DB
                }
                UriType::Artist => {
                    // fetch artist information from Spotify API and update DB
                }
                UriType::Playlist => {
                    // fetch playlist information from Spotify API and update DB
                }
                UriType::Show => {
                    // fetch show information from Spotify API and update DB
                }
                _ => {
                    return Err(DBError::InvalidURI);
                }
            }
        }

        // TODO implement
        return Ok(());
    }

    pub async fn update_now_playing(
        &self,
        user_id: &str,
        uri: &str,
        position: i64,
    ) -> Result<(), DBError> {
        // get current most recent song for user
        let res = sqlx::query(
            "SELECT item_uri, position, updated_at, CURRENT_TIMESTAMP - updated_at as time_since_update FROM play_history
            WHERE user_id = $1 ORDER BY updated_at DESC LIMIT 1",
        )
        .bind(user_id)
        .fetch_one(self.connection.as_ref().unwrap())
        .await;

        if let Ok(row) = res {
            let prev_uri: String = row.try_get("uri").unwrap_or_default();
            let prev_pos: i64 = row.try_get("position").unwrap_or(0);
            let prev_ts: i64 = row.try_get("updated_at").unwrap_or(0);
            let time_since_prev_ts: i64 = row.try_get("time_since_update").unwrap_or(0);
            if prev_uri == uri {
                let _ = sqlx::query(
                    "UPDATE play_history SET position = $1, updated_at = CURRENT_TIMESTAMP
                    WHERE user_id = $2 AND updated_at = $3",
                )
                .bind(position)
                .bind(user_id)
                .bind(prev_ts)
                .execute(self.connection.as_ref().unwrap())
                .await;

                // if pos is less than previous pos, we replayed the track
                if position < prev_pos {
                    let t = get_uri_type(uri);

                    match t {
                        UriType::Track => {
                            let _ = sqlx::query(
                                "UPDATE user_track_analytics SET replay_count = replay_count + 1
                                WHERE user_id = $1 AND track_uri = $2",
                            )
                            .bind(user_id)
                            .bind(uri)
                            .execute(self.connection.as_ref().unwrap())
                            .await;
                        }
                        UriType::Episode => {
                            let _ = sqlx::query(
                                "UPDATE user_track_episode_analytics SET replay_count = replay_count + 1
                                WHERE user_id = $1 AND episode_uri = $2"
                            )
                            .bind(user_id)
                            .bind(uri)
                            .execute(self.connection.as_ref().unwrap())
                            .await;
                        }
                        _ => {
                            return Err(DBError::InvalidURI);
                        }
                    }
                }
            } else {
                let _ = sqlx::query(
                    "INSERT INTO play_history (user_id, item_uri, position, updated_at)
                     VALUES ($1, $2, $3, CURRENT_TIMESTAMP)
                     ON CONFLICT(user_id) DO UPDATE SET
                     item_uri = excluded.uri,
                     position = excluded.position,
                     updated_at = excluded.updated_at",
                )
                .bind(user_id)
                .bind(uri)
                .bind(position)
                .execute(self.connection.as_ref().unwrap())
                .await;

                // if prev pos was small, we skipped the track
                if prev_pos < 30000 && prev_uri != "" {
                    let t = get_uri_type(&prev_uri);

                    match t {
                        UriType::Track => {
                            let _ = sqlx::query(
                                "UPDATE user_track_analytics SET skip_count = skip_count + 1
                                WHERE user_id = $1 AND track_uri = $2",
                            )
                            .bind(user_id)
                            .bind(&prev_uri)
                            .execute(self.connection.as_ref().unwrap())
                            .await;
                        }
                        UriType::Episode => {
                            let _ = sqlx::query(
                                "UPDATE user_track_episode_analytics SET skip_count = skip_count + 1
                                WHERE user_id = $1 AND episode_uri = $2"
                            )
                            .bind(user_id)
                            .bind(&prev_uri)
                            .execute(self.connection.as_ref().unwrap())
                            .await;
                        }
                        _ => {
                            return Err(DBError::InvalidURI);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
