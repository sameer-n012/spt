use crate::util::errors::DBError;
use crate::util::uri_helper::{get_uri_type, UriType};
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
                        "UPDATE user_track_analytics SET play_count = play_count + 1
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
                        "UPDATE user_track_episode_analytics SET play_count = play_count + 1
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
                        "UPDATE user_track_analytics SET skip_count = skip_count + 1
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
                        "UPDATE user_track_episode_analytics SET skip_count = skip_count + 1
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
                        "UPDATE user_track_analytics SET replay_count = replay_count + 1
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
                        "UPDATE user_track_episode_analytics SET replay_count = replay_count + 1
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
            "SELECT item_uri, position, updated_at FROM play_history
            WHERE user_id = $1 ORDER BY updated_at DESC LIMIT 1",
        )
        .bind(user_id)
        .fetch_one(self.connection.as_ref().unwrap())
        .await;

        if let Ok(row) = res {
            let prev_uri: String = row.try_get("uri").unwrap_or_default();
            let prev_pos: i64 = row.try_get("position").unwrap_or(0);
            let prev_ts: i64 = row.try_get("updated_at").unwrap_or(0);
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
