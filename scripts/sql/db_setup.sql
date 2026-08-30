CREATE TABLE IF NOT EXISTS users (
    user_id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    refresh_token TEXT,
    refresh_token_expiry TIMESTAMP,
    password_hash TEXT, -- hashed password, can be null
    salt TEXT
);

CREATE TABLE IF NOT EXISTS albums (
    uri TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    release_date TEXT NOT NULL -- date in YYYY-MM-DD format
);

CREATE TABLE IF NOT EXISTS shows (
    uri TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    publisher TEXT NOT NULL,
    description TEXT
);

CREATE TABLE IF NOT EXISTS artists (uri TEXT PRIMARY KEY, name TEXT NOT NULL);

CREATE TABLE IF NOT EXISTS playlists (
    uri TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    public BOOLEAN NOT NULL DEFAULT TRUE,
    collaborative BOOLEAN NOT NULL DEFAULT FALSE,
    snapshot_id TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (user_id)
);

CREATE TABLE IF NOT EXISTS tracks (
    uri TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    album_uri TEXT NOT NULL,
    track_number INTEGER NOT NULL,
    disc_number INTEGER NOT NULL,
    duration_ms INTEGER NOT NULL,
    explicit BOOLEAN NOT NULL DEFAULT FALSE,
    popularity INTEGER, -- score from 0 to 1000
    acousticness REAL,
    danceability REAL,
    energy REAL,
    instrumentalness REAL,
    liveness REAL,
    loudness REAL,
    speechiness REAL,
    tempo REAL,
    valence REAL,
    time_signature INTEGER, -- ranges from 3 to 7?
    pitch_class_key INTEGER, -- 0-11
    FOREIGN KEY (album_uri) REFERENCES albums (uri)
);

CREATE TABLE IF NOT EXISTS track_artists (
    track_uri TEXT NOT NULL,
    artist_uri TEXT NOT NULL,
    PRIMARY KEY (track_uri, artist_uri),
    FOREIGN KEY (track_uri) REFERENCES tracks (uri),
    FOREIGN KEY (artist_uri) REFERENCES artists (uri)
);

CREATE TABLE IF NOT EXISTS episodes (
    uri TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    show_uri TEXT NOT NULL,
    episode_number INTEGER NOT NULL,
    release_date DATE NOT NULL, -- date in YYYY-MM-DD format
    description TEXT,
    duration_ms INTEGER NOT NULL,
    explicit BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (show_uri) REFERENCES shows (uri)
);

CREATE TABLE IF NOT EXISTS playlist_contents (
    playlist_uri TEXT NOT NULL,
    item_uri TEXT,
    episode_uri TEXT,
    position INTEGER NOT NULL,
    added_at TEXT, -- date in YYYY-MM-DD format
    added_by TEXT, -- user id, not necessarily in db
    PRIMARY KEY (playlist_uri, item_uri, episode_uri),
    FOREIGN KEY (playlist_uri) REFERENCES playlists (uri),
    FOREIGN KEY (item_uri) REFERENCES tracks (uri),
    FOREIGN KEY (episode_uri) REFERENCES episodes (uri)
);

-- Per-user play/skip/replay counters for tracks. Table name matches what
-- db_manager.rs's increment_play_count/increment_skip_count/increment_replay_count/
-- update_now_playing actually query at runtime.
CREATE TABLE IF NOT EXISTS user_track_analytics (
    user_id TEXT NOT NULL,
    track_uri TEXT NOT NULL,
    play_count INTEGER DEFAULT 0,
    skip_count INTEGER DEFAULT 0,
    replay_count INTEGER DEFAULT 0,
    last_played TIMESTAMP,
    PRIMARY KEY (user_id, track_uri),
    FOREIGN KEY (user_id) REFERENCES users (user_id),
    FOREIGN KEY (track_uri) REFERENCES tracks (uri)
);

-- Same as user_track_analytics, but for episodes. Kept as a separate table
-- (rather than nullable track_uri/episode_uri columns on one table) so the
-- primary key can stay a simple (user_id, episode_uri) pair.
CREATE TABLE IF NOT EXISTS user_track_episode_analytics (
    user_id TEXT NOT NULL,
    episode_uri TEXT NOT NULL,
    play_count INTEGER DEFAULT 0,
    skip_count INTEGER DEFAULT 0,
    replay_count INTEGER DEFAULT 0,
    last_played TIMESTAMP,
    PRIMARY KEY (user_id, episode_uri),
    FOREIGN KEY (user_id) REFERENCES users (user_id),
    FOREIGN KEY (episode_uri) REFERENCES episodes (uri)
);

-- Tracks only the current/last-known playback position per user. db_manager.rs's
-- update_now_playing does `INSERT ... ON CONFLICT(user_id) DO UPDATE`, which requires
-- a unique constraint on user_id alone, so this is a single latest-position row per
-- user rather than a full history log.
CREATE TABLE IF NOT EXISTS play_history (
    user_id TEXT PRIMARY KEY,
    item_uri TEXT NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    position INTEGER NOT NULL, -- position in ms
    FOREIGN KEY (user_id) REFERENCES users (user_id)
);

-- Caches the Spotify OAuth tokens obtained for a given Spotify app (client_id) so a
-- local dev/test server restart can resume from the stored refresh_token instead of
-- re-running the browser-based authorization flow. Keyed by the Spotify application's
-- client_id (from SPT_SPOTIFY_API_CLIENT_ID), not by a Spotify user id, since this proxy
-- never fetches /me and only handles one authorized account per app registration.
CREATE TABLE IF NOT EXISTS cached_auth_tokens (
    spotify_client_id TEXT PRIMARY KEY,
    access_token TEXT,
    access_token_expiry INTEGER, -- unix timestamp (seconds), null if unknown/expired
    refresh_token TEXT NOT NULL,
    updated_at INTEGER NOT NULL -- unix timestamp (seconds)
);
