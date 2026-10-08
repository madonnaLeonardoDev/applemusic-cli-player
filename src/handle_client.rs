
use thirtyfour::WebDriver;
use tokio::net::UnixStream;
use tokio::io::{AsyncBufReadExt,BufReader};
use crate::apple_music_navigation::ItemType;
use crate::daemon_controls::*;
use clap::{Parser, Subcommand};
use crate::paths::NAME;

#[derive(Parser)]
#[command(name = "daemon-cli", author, version, about = "Controls Apple Music daemon")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(visible_alias = "s")]
    Search {
        #[arg(short = 't', long)]
        stype: String,

        #[arg(short, long)]
        query: String,

        #[arg(short, long)]
        next: bool,

        #[arg(short, long)]
        play: bool,

        #[arg(short, long)]
        library: bool
    },
    #[command(visible_alias = "pid")]
    Playid {
        #[arg(short = 't', long)]
        stype: String,
        #[arg(short, long)]
        id: String,
        #[arg(short, long)]
        play: bool
    },
    #[command(visible_alias = "nx")]
    Next {},
    #[command(visible_alias = "pv")]
    Prev {},
    #[command(visible_alias = "pp")]
    PlayPause{},
    #[command(visible_alias = "sh")]
    Shuffle{},
    #[command(visible_alias = "c")]
    Current{},
    #[command(visible_alias = "lspl")]
    LsPlaylists{}
}

pub async fn handle_client(stream: &mut UnixStream, driver: &WebDriver) -> Result<String, String> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    let bytes_read = reader.read_line(&mut line).await
    .map_err(|e| format!("Could not read stream: {}", e))?;

    if bytes_read <= 0 {
        return Err("Invalid empty command".to_string());
    }

    let raw_args = line.trim().split_whitespace();
    let args = std::iter::once(NAME).chain(raw_args);

    let cli = match Cli::try_parse_from(args) {
        Ok(c) => c,
        Err(e) => return Ok(e.to_string()), 
    };
    



    let result: Result<String, String> =  match cli.command {
        Commands::Search { stype, query, next, play, library } => {
            let search_type = match stype.to_lowercase().as_str() {
                "song" | "s" => ItemType::Song,
                "album" | "a" => ItemType::Album,
                "playlist" | "p" => ItemType::Playlist,
                _ => {
                   return Err("Invalid search type (stype) argument".to_string());
                }
            };
            let joined_query = query.replace("-", " ");
            Ok(cmd_search(driver, joined_query, &search_type, library, next, play).await?)
        },
        Commands::Current {  } => {
            Ok(cmd_current_track(driver).await?)
        },
        Commands::LsPlaylists {  } => {
            Ok(cmd_list_playlists(driver).await?)
        },
        Commands::Playid { stype, id, play } => {
            let search_type = match stype.to_lowercase().as_str() {
                "song" | "s" => ItemType::Song,
                "album" | "a" => ItemType::Album,
                "playlist" | "p" => ItemType::Playlist,
                _ => {
                    return Err("Invalid search type (stype) argument".to_string());
                }
            };

            Ok(cmd_play_by_id(driver, id, search_type, play).await?)
        },
        Commands::PlayPause {  } => {
            Ok(cmd_play_pause(driver).await?)
        },
        Commands::Next {  } => {
            Ok(cmd_next(driver).await?)
        },
        Commands::Prev {  } => {
            Ok(cmd_prev(driver).await?)
        },
        Commands::Shuffle {  } => {
            Ok(cmd_toggle_shuffle(driver).await?)
        }
    };

    if let Err(e) = result {
        return Ok(e.to_string());
    }

    Ok(result.unwrap())

}