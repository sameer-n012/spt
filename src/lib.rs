pub mod server {
    pub mod web {
        pub mod routes;
        pub mod server;
        pub mod spt_api_proxy;
    }
    pub mod db {
        pub mod db_manager;
        pub mod transaction_manager;
    }
}

pub mod models {
    pub mod album;
    pub mod artist;
    pub mod episode;
    pub mod playlist;
    pub mod show;
    pub mod track;
    pub mod user;
}

pub mod util {
    pub mod errors;
    pub mod uri_helper;
}

pub mod client {
    pub mod local_api_proxy;
    pub mod cli {
        pub mod cli_app;
        pub mod eval;
        pub mod formatter;
        pub mod parser;
    }
    pub mod core {
        pub mod playback_manager;
        pub mod playlist_manager;
        pub mod search_manager;
        pub mod status_manager;
    }
}
