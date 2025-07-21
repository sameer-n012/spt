use crate::client::cli::eval::eval;
use crate::client::cli::parser::{parse, tokenize, verify_command, verify_flags, ParseError};
use crate::client::local_api_proxy::ApiProxy;
use log::{debug, info};

use std::collections::{HashMap, HashSet};
// use crate::core::playlist_manager::PlaylistManager;
// use crate::core::queue_manager::QueueManager;
// use crate::core::search_manager::SearchManager;
// use crate::core::status_manager::StatusManager;
// use crate::core::transaction_manager::TransactionManager;
// use clap::{Parser, Subcommand};

// #[derive(Parser)]
// #[clap(name = "spt")]
// #[command(about = "Spotify terminal application", version = "0.1")]

// pub struct Cli {
//     #[command(subcommand)]
//     pub command: Commands,
// }

// #[derive(Subcommand)]
// pub enum Commands {
//     Play,  // sets playback to play
//     Pause, // sets playback to pause
//     Previous {
//         n: Option<u8>, // if not None, skips to nth previous track (None = 1)
//     }, // skips to nth previous track
//     Next {
//         n: Option<u8>, // if not None, skips to nth next track (None = 1)
//     }, // skips to nth next track
//     Volume {
//         level: Option<u8>, // if not None, desired volume level
//     }, // sets volume to level or gets volume level
//     Device {
//         name: String,
//     }, // sets the playback device by name
//     User,  // gets information about the current user
//     Now {
//         h: Option<bool>, // if true, human readable (track name, artist, album)
//     }, // gets now playing information
//     Devices {
//         h: Option<bool>, // if true, human readable (device names)
//     }, // gets a list of available devices
//     Playlists {
//         uris: String,    // the URIs of the playlists
//         i: Option<bool>, // if true, internal representation (playlist/track IDs)
//         h: Option<bool>, // if true, human readable (playlist names, tracks)
//     }, // gets a list of playlists, shows tracks if URIs were provided
//     PlaylistAdd {
//         name: String, // the name of the playlist
//         uris: String, // the URIs of the tracks to add
//     }, // adds tracks to a playlist
//     PlaylistRemove {
//         name: String, // the name of the playlist
//         uris: String, // the URIs of the tracks to remove
//     }, // removes tracks from a playlist
//     PlaylistCreate {
//         name: String, // the name of the new playlist
//         uris: String, // the URIs of the tracks to add
//     }, // creates a new playlist with tracks
//     PlaylistDelete {
//         name: String, // the name of the playlist to delete
//     },
//     Search {
//         query: String,   // the search query
//         i: Option<bool>, // if true, internal representation (track IDs)
//         h: Option<bool>, // if true, human readable (track names, albums, artists)
//     },
//     Describe {
//         uris: String, // the URIs of the tracks to describe
//     }, // describes tracks by URIs
//     Filter {
//         query: String, // the filter query
//         uris: String,  // the URIs of the tracks to filter
//     },
//     Queue {
//         uris: Option<Vec<String>>, // if not None, queue tracks by URIs, show queue
//         h: Option<bool>,           // if true, human readable (track names, artists)
//     }, // queues tracks by URIs or shows the current queue
//     Recent {
//         n: Option<u8>,   // if not None, number of recent tracks to show (None = 20)
//         h: Option<bool>, // if true, human readable (track names, artists)
//     }, // gets recent history
// }

// pub async fn run_cli(api_proxy: &mut ApiProxy) {
//     let cli = Cli::parse();

//     let mut playback_manager = PlaybackManager::new(api_proxy);
//     // let mut queue_manager = QueueManager::new();
//     // let mut status_manager = StatusManager::new();
//     // let mut search_manager = SearchManager::new();
//     // let mut playlist_manager = PlaylistManager::new();
//     // let mut transaction_manager = TransactionManager::new();
//     //
//     let mut output: Option<String> = None;

//     match &cli.command {
//         // Playback Control
//         Commands::Play => {
//             output = playback_manager.play().await;
//         }
//         Commands::Pause => {
//             output = playback_manager.pause().await;
//         }
//         Commands::Next { n } => {
//             output = playback_manager.next(n.unwrap_or(1)).await;
//         }
//         Commands::Previous { n } => {
//             output = playback_manager.previous(n.unwrap_or(1)).await;
//         }
//         Commands::Volume { level } => {
//             if level.is_none() {
//                 output = playback_manager.get_volume().await;
//             } else {
//                 output = playback_manager.set_volume(level.unwrap()).await;
//             }
//         }
//         Commands::Device { name } => {
//             output = playback_manager.device(name).await;
//         }
//         Commands::Devices { h } => {
//             output = playback_manager.devices(h.unwrap_or(false)).await;
//         }
//         Commands::Now { h } => {
//             output = playback_manager.now(h.unwrap_or(false)).await;
//         }
//         Commands::Queue { uris, h } => {
//             if uris.is_none() {
//                 output = playback_manager.queue(h.unwrap_or(false)).await;
//             } else {
//                 output = playback_manager.queue_add(uris.to_owned().unwrap()).await;
//             }
//         }
//         Commands::Recent { n, h } => {
//             output = playback_manager
//                 .recent(n.unwrap_or(20), h.unwrap_or(false))
//                 .await;
//         }

//         // Status Information
//         Commands::User => {
//             //status_manager.user();
//         }

//         // Playlist Management
//         Commands::Playlists { uris, i, h } => {
//             // playlist_manager.playlists();
//         }
//         Commands::PlaylistAdd { name, uris } => {
//             //playlist_manager.playlist_add(name, uris);
//         }
//         Commands::PlaylistRemove { name, uris } => {
//             //playlist_manager.playlist_remove(name, uris);
//         }
//         Commands::PlaylistCreate { name, uris } => {
//             //playlist_manager.playlist_create(name, uris);
//         }
//         Commands::PlaylistDelete { name } => {
//             //playlist_manager.playlist_delete(name);
//         }

//         // Search
//         Commands::Search { query, i, h } => {
//             // search_manager.search(query);
//         }
//         Commands::Describe { uris } => {
//             // search_manager.describe(uris);
//         }
//         Commands::Filter { query, uris } => {
//             // search_manager.filter(query, uris);
//         }
//     }

//     if output.is_some() {
//         println!("{}", output.unwrap());
//     }
// }

async fn run(
    api_proxy: &mut ApiProxy,
    input: String,
    flags: HashMap<String, Vec<String>>,
) -> Result<Option<String>, ParseError> {
    let command_list: HashSet<String> = HashSet::from_iter(flags.keys().map(|s| s.to_string()));

    debug!("Received command {}", input);

    let tokens = tokenize(&input)?;
    verify_command(&tokens, &command_list)?;

    let cmd = parse(&tokens, &command_list)?;
    verify_flags(&cmd, &flags)?;

    debug!("Tokenized and parsed {:?}", tokens);

    return Ok(eval(api_proxy, &cmd).await);
}

pub async fn run_cli(api_proxy: &mut ApiProxy, args: Vec<String>) {
    let mut flags = HashMap::new();
    flags.insert("play".to_string(), vec![]);
    flags.insert("pause".to_string(), vec![]);
    flags.insert("next".to_string(), vec![]);
    flags.insert("previous".to_string(), vec![]);
    flags.insert("volume".to_string(), vec![]);
    flags.insert("device".to_string(), vec![]);
    flags.insert("devices".to_string(), vec!["-h".to_string()]);
    flags.insert("now".to_string(), vec!["-h".to_string()]);
    flags.insert("queue".to_string(), vec!["-h".to_string()]);
    flags.insert("recent".to_string(), vec!["-h".to_string()]);

    let input = args[1..].join(" ");
    let output = run(api_proxy, input, flags).await.unwrap_or(None);

    if output.is_some() {
        println!("{}", output.unwrap());
    }
}
