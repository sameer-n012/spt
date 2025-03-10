use crate::client::core::playback_manager::PlaybackManager;
use crate::client::local_api_manager::ApiManager;
// use crate::core::playlist_manager::PlaylistManager;
// use crate::core::queue_manager::QueueManager;
// use crate::core::search_manager::SearchManager;
// use crate::core::status_manager::StatusManager;
// use crate::core::transaction_manager::TransactionManager;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "spt")]
#[command(about = "Spotify terminal application", version = "0.1")]

pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Play,  // sets playback to play
    Pause, // sets playback to pause
    Previous {
        n: Option<u8>,
    }, // skips to nth previous track
    Next {
        n: Option<u8>,
    }, // skips to nth next track
    Volume {
        level: Option<u8>,
    }, // sets volume to level or gets volume level
    Device {
        name: String,
    },
    User,
    Now,
    Devices {
        h: Option<bool>,
    },
    Playlists {
        i: Option<bool>,
        h: Option<bool>,
    },
    Playlist {
        uris: String,
        i: Option<bool>,
        h: Option<bool>,
    },
    PlaylistAdd {
        name: String,
        uris: String,
    },
    PlaylistRemove {
        name: String,
        uris: String,
    },
    PlaylistCreate {
        name: String,
        uris: String,
    },
    PlaylistDelete {
        name: String,
    },
    Search {
        query: String,
        i: Option<bool>,
        h: Option<bool>,
    },
    Describe {
        uris: String,
    },
    Queue {
        uris: String,
    },
    Clear,
    Push {
        uris: String,
    },
    Filter {
        query: String,
        uris: String,
    },
}

pub async fn run_cli(api_manager: &mut ApiManager) {
    let cli = Cli::parse();

    let mut playback_manager = PlaybackManager::new(api_manager);
    // let mut queue_manager = QueueManager::new();
    // let mut status_manager = StatusManager::new();
    // let mut search_manager = SearchManager::new();
    // let mut playlist_manager = PlaylistManager::new();
    // let mut transaction_manager = TransactionManager::new();
    //
    let mut output: Option<String> = None;

    match &cli.command {
        // Playback Control
        Commands::Play => {
            playback_manager.play();
        }
        Commands::Pause => {
            playback_manager.pause();
        }
        Commands::Next { n } => {
            if n.is_none() {
                playback_manager.next(1);
            } else {
                playback_manager.next(n.unwrap());
            }
        }
        Commands::Previous { n } => {
            if n.is_none() {
                playback_manager.previous(1);
            } else {
                playback_manager.previous(n.unwrap());
            }
        }
        Commands::Volume { level } => {
            if level.is_none() {
                playback_manager.get_volume();
            } else {
                playback_manager.set_volume(level.unwrap());
            }
        }
        Commands::Device { name } => {
            playback_manager.device(name);
        }

        // Status Information
        Commands::User => {
            //status_manager.user();
        }
        Commands::Devices { h } => {
            //status_manager.devices();
        }
        Commands::Now => {
            output = Some(playback_manager.now().await);
        }

        // Playlist Management
        Commands::Playlists { i, h } => {
            // playlist_manager.playlists();
        }
        Commands::Playlist { uris, i, h } => {
            //playlist_manager.playlist(uris);
        }
        Commands::PlaylistAdd { name, uris } => {
            //playlist_manager.playlist_add(name, uris);
        }
        Commands::PlaylistRemove { name, uris } => {
            //playlist_manager.playlist_remove(name, uris);
        }
        Commands::PlaylistCreate { name, uris } => {
            //playlist_manager.playlist_create(name, uris);
        }
        Commands::PlaylistDelete { name } => {
            //playlist_manager.playlist_delete(name);
        }

        // Search
        Commands::Search { query, i, h } => {
            // search_manager.search(query);
        }
        Commands::Describe { uris } => {
            // search_manager.describe(uris);
        }
        Commands::Filter { query, uris } => {
            // search_manager.filter(query, uris);
        }

        // Queue Management
        Commands::Queue { uris } => {
            // queue_manager.queue(uris);
        }
        Commands::Clear => {
            // queue_manager.clear();
        }

        // Transaction Management
        Commands::Push { uris } => {
            // transaction_manager.push(uris);
        }
    }

    if output.is_some() {
        println!("{}", output.unwrap());
    }
}
