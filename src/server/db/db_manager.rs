use crate::util::errors::DBError;
use crate::util::uri_helper::{get_uri_type, UriType};
use sqlx::Column;
use sqlx::Row;
use std::str::FromStr;

use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};

// schema embedded at compile time so the server does not depend on cwd
const DB_SETUP_SQL: &str = include_str!("../../../scripts/sql/db_setup.sql");

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

    // DB Caching: Get/Insert Methods for Each Item Type

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

    // --- Cache population from live Spotify API responses ---
    //
    // Called after a cache-miss fetch (see routes.rs's per-item lookup routes) to store the
    // freshly-fetched item so the next lookup can be served from SQLite instead of hitting
    // Spotify again. Best-effort: a mapping/insert failure here should never fail the request
    // that's already carrying a perfectly good live API response back to the caller.
    //
    // Spotify's response shapes don't line up 1:1 with this project's flatter DB schema (nested
    // album/artist/show objects, renamed fields like album_type vs. this schema's `type`), so
    // each cache_*_from_api method below re-maps fields before handing them to the existing
    // Model::from_api_record()/generate_insert_query() pair.

    pub async fn cache_item_from_api(
        &self,
        uri: &str,
        api_json: &serde_json::Value,
    ) -> Result<(), DBError> {
        match get_uri_type(uri) {
            UriType::Track => self.cache_track_from_api(api_json).await,
            UriType::Album => self.cache_album_from_api(api_json).await,
            UriType::Artist => self.cache_artist_from_api(api_json).await,
            UriType::Episode => self.cache_episode_from_api(api_json).await,
            UriType::Show => self.cache_show_from_api(api_json).await,
            UriType::Playlist => self.cache_playlist_from_api(api_json).await,
            UriType::Unknown => Err(DBError::InvalidURI),
        }
    }

    async fn cache_track_from_api(&self, json: &serde_json::Value) -> Result<(), DBError> {
        // Cache the embedded SimplifiedAlbumObject/SimplifiedArtistObjects first (best-effort —
        // a failure here shouldn't stop the track itself from being cached).
        let _ = self.cache_album_from_api(&json["album"]).await;
        if let Some(artists) = json["artists"].as_array() {
            for artist_json in artists {
                let _ = self.cache_artist_from_api(artist_json).await;
            }
        }

        let mapped = serde_json::json!({
            "uri": json["uri"].clone(),
            "name": json["name"].clone(),
            "album_uri": json["album"]["uri"].clone(),
            "track_number": json["track_number"].clone(),
            "disc_number": json["disc_number"].clone(),
            "duration_ms": json["duration_ms"].clone(),
            "explicit": json["explicit"].clone(),
            "popularity": json["popularity"].clone(), // deprecated field; commonly absent/null
        });

        let track = crate::models::track::Track::from_api_record(mapped, None)
            .map_err(|_| DBError::CacheParseError)?;

        track
            .generate_insert_query()
            .execute(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?;

        if let Some(artists) = json["artists"].as_array() {
            for artist_json in artists {
                if let Some(artist_uri) = artist_json["uri"].as_str() {
                    let _ = sqlx::query(
                        "INSERT INTO track_artists (track_uri, artist_uri) VALUES ($1, $2)",
                    )
                    .bind(&track.uri)
                    .bind(artist_uri)
                    .execute(self.connection.as_ref().unwrap())
                    .await;
                }
            }
        }

        Ok(())
    }

    async fn cache_album_from_api(&self, json: &serde_json::Value) -> Result<(), DBError> {
        let mapped = serde_json::json!({
            "uri": json["uri"].clone(),
            "name": json["name"].clone(),
            "type": json["album_type"].clone(), // Spotify calls this `album_type`, not `type`
            "release_date": json["release_date"].clone(),
        });

        let album = crate::models::album::Album::from_api_record(mapped)
            .map_err(|_| DBError::CacheParseError)?;

        album
            .generate_insert_query()
            .execute(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?;

        Ok(())
    }

    async fn cache_artist_from_api(&self, json: &serde_json::Value) -> Result<(), DBError> {
        // Spotify's ArtistObject already has flat uri/name fields matching this model directly.
        let artist = crate::models::artist::Artist::from_api_record(json.clone())
            .map_err(|_| DBError::CacheParseError)?;

        artist
            .generate_insert_query()
            .execute(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?;

        Ok(())
    }

    async fn cache_show_from_api(&self, json: &serde_json::Value) -> Result<(), DBError> {
        // Spotify's Show/SimplifiedShowObject already has flat uri/name/publisher/description
        // fields matching this model directly.
        let show = crate::models::show::Show::from_api_record(json.clone())
            .map_err(|_| DBError::CacheParseError)?;

        show.generate_insert_query()
            .execute(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?;

        Ok(())
    }

    async fn cache_episode_from_api(&self, json: &serde_json::Value) -> Result<(), DBError> {
        // Cache the embedded SimplifiedShowObject first (best-effort).
        let _ = self.cache_show_from_api(&json["show"]).await;

        let mapped = serde_json::json!({
            "uri": json["uri"].clone(),
            "name": json["name"].clone(),
            "show_uri": json["show"]["uri"].clone(),
            // Spotify's episode objects don't expose a stored episode-number field the way this
            // schema expects one; default to 0 rather than dropping the episode from the cache.
            "episode_number": if json["episode_number"].is_null() {
                serde_json::json!(0)
            } else {
                json["episode_number"].clone()
            },
            "release_date": json["release_date"].clone(),
            "description": json["description"].clone(),
            "duration_ms": json["duration_ms"].clone(),
            "explicit": json["explicit"].clone(),
        });

        let episode = crate::models::episode::Episode::from_api_record(mapped, None)
            .map_err(|_| DBError::CacheParseError)?;

        episode
            .generate_insert_query()
            .execute(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?;

        Ok(())
    }

    async fn cache_playlist_from_api(&self, json: &serde_json::Value) -> Result<(), DBError> {
        let owner_id = json["owner"]["id"].as_str().unwrap_or_default().to_string();
        let display_name = json["owner"]["display_name"]
            .as_str()
            .unwrap_or(&owner_id)
            .to_string();

        // a signed-in owner already has a row keyed by account_id, else key by spotify id
        let user_id: String = sqlx::query_scalar("SELECT user_id FROM users WHERE spotify_id = $1")
            .bind(&owner_id)
            .fetch_optional(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?
            .unwrap_or(owner_id);

        // playlists.user_id has no other source of truth here; ensure a minimal placeholder
        // user row exists for the owner so caching a playlist doesn't depend on its owner ever
        // having authorized this app themselves.
        let _ = sqlx::query(
            "INSERT INTO users (user_id, display_name) VALUES ($1, $2) \
                ON CONFLICT(user_id) DO NOTHING",
        )
        .bind(&user_id)
        .bind(&display_name)
        .execute(self.connection.as_ref().unwrap())
        .await;

        let mapped = serde_json::json!({
            "uri": json["uri"].clone(),
            "user_id": user_id,
            "name": json["name"].clone(),
            "description": json["description"].clone(),
            "public": json["public"].clone(),
            "collaborative": json["collaborative"].clone(),
            "snapshot_id": json["snapshot_id"].clone(),
        });

        let playlist = crate::models::playlist::Playlist::from_api_record(mapped)
            .map_err(|_| DBError::CacheParseError)?;

        playlist
            .generate_insert_query()
            .execute(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?;

        Ok(())
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

        let opts = SqliteConnectOptions::from_str(&db_addr)
            .map_err(|_| DBError::ConnectionError)?
            .create_if_missing(true);

        let conn = SqlitePool::connect_with(opts)
            .await
            .map_err(|_| DBError::ConnectionError)?;

        self.connection = Some(conn);

        // runs on every start, contains only CREATE TABLE IF NOT EXISTS so safe
        // & adds new tables to a DB file that was made with an older schema.
        sqlx::query(DB_SETUP_SQL)
            .execute(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::SetupError)?;

        // users.spotify_id was added later, CREATE TABLE IF NOT EXISTS won't add it to old DBs
        let has_spotify_id: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM pragma_table_info('users') WHERE name = 'spotify_id'",
        )
        .fetch_one(self.connection.as_ref().unwrap())
        .await
        .map_err(|_| DBError::SetupError)?;
        if has_spotify_id == 0 {
            sqlx::query("ALTER TABLE users ADD COLUMN spotify_id TEXT")
                .execute(self.connection.as_ref().unwrap())
                .await
                .map_err(|_| DBError::SetupError)?;
        }
        sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS users_spotify_id ON users (spotify_id)")
            .execute(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::SetupError)?;

        Ok(())
    }

    // upserts the signed-in user keyed by account_id
    pub async fn save_signed_in_user(
        &self,
        account_id: &str,
        spotify_id: &str,
        display_name: &str,
    ) -> Result<(), DBError> {
        sqlx::query(
            "INSERT INTO users (user_id, spotify_id, display_name) VALUES ($1, $2, $3) \
                ON CONFLICT(user_id) DO UPDATE SET \
                spotify_id = excluded.spotify_id, display_name = excluded.display_name",
        )
        .bind(account_id)
        .bind(spotify_id)
        .bind(display_name)
        .execute(self.connection.as_ref().unwrap())
        .await
        .map_err(|_| DBError::QueryError)?;

        // playlists cached before sign-in point at the owner placeholder row (keyed by spotify id)
        sqlx::query("UPDATE playlists SET user_id = $1 WHERE user_id = $2")
            .bind(account_id)
            .bind(spotify_id)
            .execute(self.connection.as_ref().unwrap())
            .await
            .map_err(|_| DBError::QueryError)?;

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

    pub async fn update_track_information(&self, _uri: &str) -> Result<(), DBError> {
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
            "SELECT item_uri, position, updated_at FROM play_history
            WHERE user_id = $1 ORDER BY updated_at DESC LIMIT 1",
        )
        .bind(user_id)
        .fetch_one(self.connection.as_ref().unwrap())
        .await;

        if let Ok(row) = res {
            // updated_at is a SQLite TIMESTAMP column, populated via CURRENT_TIMESTAMP, so it
            // comes back as text (e.g. "2026-08-30 12:34:56"), not an integer.
            let prev_uri: String = row.try_get("item_uri").unwrap_or_default();
            let prev_pos: i64 = row.try_get("position").unwrap_or(0);
            let prev_ts: String = row.try_get("updated_at").unwrap_or_default();
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
                     item_uri = excluded.item_uri,
                     position = excluded.position,
                     updated_at = excluded.updated_at",
                )
                .bind(user_id)
                .bind(uri)
                .bind(position)
                .execute(self.connection.as_ref().unwrap())
                .await;

                // if prev pos was small, we skipped the track
                if prev_pos < 30000 && !prev_uri.is_empty() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn setup_adds_missing_tables_to_existing_db() {
        let path = std::env::temp_dir().join(format!("spt_setup_test_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let url = path.to_str().unwrap().to_string();

        let mut db = DBManager::new(url.clone(), 0);
        db.setup().await.unwrap();
        let pool = db.connection.as_ref().unwrap();
        sqlx::query("DROP TABLE cached_auth_tokens")
            .execute(pool)
            .await
            .unwrap();
        pool.close().await;

        let mut db = DBManager::new(url.clone(), 0);
        db.setup().await.unwrap();
        db.setup().await.unwrap();
        db.save_cached_auth_tokens("app", None, None, "refresh", 0)
            .await
            .unwrap();
        let cached = db.load_cached_auth_tokens("app").await.unwrap();
        assert_eq!(cached, Some((None, None, "refresh".to_string())));

        db.connection.as_ref().unwrap().close().await;
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn setup_adds_spotify_id_to_old_users_table() {
        let path = std::env::temp_dir().join(format!("spt_users_test_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let url = path.to_str().unwrap().to_string();

        // users table as created by the schema before spotify_id existed
        let opts = SqliteConnectOptions::from_str(&url)
            .unwrap()
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(opts).await.unwrap();
        sqlx::query("CREATE TABLE users (user_id TEXT PRIMARY KEY, display_name TEXT NOT NULL)")
            .execute(&pool)
            .await
            .unwrap();
        pool.close().await;

        let mut db = DBManager::new(url, 0);
        db.setup().await.unwrap();
        db.setup().await.unwrap();
        db.save_signed_in_user("acc", "sid", "Me").await.unwrap();

        db.connection.as_ref().unwrap().close().await;
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn signed_in_user_owns_own_playlists() {
        let path = std::env::temp_dir().join(format!("spt_owner_test_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut db = DBManager::new(path.to_str().unwrap().to_string(), 0);
        db.setup().await.unwrap();

        let playlist = |uri: &str| {
            serde_json::json!({
                "uri": uri, "name": "P", "public": true, "collaborative": false, "snapshot_id": "s",
                "owner": {"id": "sid", "display_name": "Me"}
            })
        };
        let owner = |uri: &str| {
            sqlx::query_scalar::<_, String>("SELECT user_id FROM playlists WHERE uri = $1")
                .bind(uri.to_string())
                .fetch_one(db.connection.as_ref().unwrap())
        };

        // cached before sign-in: placeholder owner, then moved to the account row on sign-in
        db.cache_playlist_from_api(&playlist("spotify:playlist:p1"))
            .await
            .unwrap();
        assert_eq!(owner("spotify:playlist:p1").await.unwrap(), "sid");
        db.save_signed_in_user("acc", "sid", "Me").await.unwrap();
        assert_eq!(owner("spotify:playlist:p1").await.unwrap(), "acc");

        // cached after sign-in: uses the account row directly
        db.cache_playlist_from_api(&playlist("spotify:playlist:p2"))
            .await
            .unwrap();
        assert_eq!(owner("spotify:playlist:p2").await.unwrap(), "acc");

        db.connection.as_ref().unwrap().close().await;
        let _ = std::fs::remove_file(&path);
    }
}
