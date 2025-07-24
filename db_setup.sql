CREATE TABLE IF NOT EXISTS users (
    user_id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    refresh_token TEXT,
    refresh_token_expiry TIMESTAMP,
    password_hash TEXT, -- hashed password, can be null
    salt TEXT,
);

CREATE TABLE IF NOT EXISTS albums (
    uri TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    release_date TEXT NOT NULL, -- date in YYYY-MM-DD format
);

CREATE TABLE IF NOT EXISTS shows (
    uri TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    publisher TEXT NOT NULL,
    description TEXT,
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
    artist_uri TEXT NOT NULL, -- comma separated list of artist URIs
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
    score REAL, -- score heuristic
    FOREIGN KEY (album_uri) REFERENCES albums (uri)
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
    score REAL, -- score heuristic
    FOREIGN KEY (show_uri) REFERENCES shows (uri)
);

CREATE TABLE IF NOT EXISTS playlist_contents (
    playlist_uri TEXT NOT NULL,
    item_uri TEXT,
    episode_uri TEXT,
    position INTEGER NOT NULL,
    added_at TEXT, -- date in YYYY-MM-DD format
    added_by TEXT, -- user id, not necessarily in db
    PRIMARY KEY (playlist_uri, track_uri, episode_uri),
    FOREIGN KEY (playlist_uri) REFERENCES playlists (uri),
    FOREIGN KEY (track_uri) REFERENCES tracks (uri),
    FOREIGN KEY (episode_uri) REFERENCES episodes (uri)
)
