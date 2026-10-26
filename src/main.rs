use std::os::unix::net::UnixStream;
use std::io::{Write, Read};
use std::net::Shutdown;
use clap::{Parser, Subcommand};
use wincode_derive::{SchemaRead, SchemaWrite};
use crate::paths::{DAEMON_SOCKET_PATH};
use crate::daemon::{kill_all, start_daemon};
use crate::display_mode::run_player_loop;
use wincode;
mod daemon;
mod handle_client;
mod paths;
mod browser;
mod apple_music_navigation;
mod daemon_controls;
mod display_mode;

#[derive(Parser, SchemaWrite, SchemaRead)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, SchemaWrite, SchemaRead)]
pub enum Commands {

    ///Start the background services that handle the music player
    #[command(visible_alias = "sd")]
    Startd{
        #[arg(short = 'H', long)]
        head:bool
    },

    ///Kill all the services that handle the music player
    #[command(visible_alias = "kd")]
    Killd,

    ///Search a song, album, or playlist by query
    #[command(visible_alias = "s")]
    Search {

        ///Specify the targets type (a, album | p, playlist | s, song)
        #[arg(short = 't', long)]
        stype: String,

        ///Search query
        #[arg(short, long)]
        query: String,

        ///Play Next
        #[arg(short, long)]
        next: bool,

        ///Play now
        #[arg(short = 'p', long = "instaplay")]
        insta_play: bool,

        ///Search in your library
        #[arg(short, long)]
        library: bool
    },

    ///Play by id
    #[command(visible_alias = "pid")]
    Playid {
        ///Specify tracks type
        #[arg(short = 't', long)]
        stype: String,

        ///Items ID
        #[arg(short, long)]
        id: String,

        ///Play Next
        #[arg(short, long)]
        next: bool
    },

    ///Next Song
    #[command(visible_alias = "nx")]
    Next,

    ///Previous Song
    #[command(visible_alias = "pv")]
    Prev,

    ///Toggle play
    #[command(visible_alias = "pp")]
    PlayPause,

    ///Shuffle queue
    #[command(visible_alias = "sh")]
    Shuffle,

    ///Get current track playing
    #[command(visible_alias = "c")]
    Current,

    ///List all library playlists
    #[command(visible_alias = "lspl")]
    LsPlaylists,

    ///Clear the queue
    #[command(visible_alias = "cq")]
    ClearQueue,

    ///List the queue
    #[command(visible_alias = "lq")]
    LsQueue,

    ///Get or Adjust volume
    #[command(visible_alias = "v")]
    Volume {
        #[arg(
            trailing_var_arg = true,
            allow_hyphen_values = true,
            num_args = 0..=1
        )]
        args: Option<u32>,
    },

    ///Graphical music deck (requires terminal with image rendering)
    #[command(visible_alias = "d")]
    Display,

    //INTERNAL COMMANDS
    #[command(hide = true)]
    GetPlayingTrackMeta
}

fn cmd_to_daemon(cli: Cli) {
            let cmd_as_bytes= wincode::serialize(&cli.command).unwrap();
            let payload_len = cmd_as_bytes.len() as u32;
            match UnixStream::connect(DAEMON_SOCKET_PATH.as_path()) {
                Ok(mut stream) => {
                    if let Err(e) = stream.write_all(&payload_len.to_le_bytes()) {
                        println!("Error writing length prefix to stream: {}", e);
                        return;
                    }

                    if let Err(e) = stream.write_all(&cmd_as_bytes) {
                        println!("Error writing payload to stream: {}", e);
                        return;
                    }

                    if let Err(e) = stream.shutdown(Shutdown::Write) {
                        println!("Error on stream shutdown: {}", e);
                        return;
                    }

                    let mut response = String::new();
                    if let Ok(_) = stream.read_to_string(&mut response) {
                        print!("{}", response);
                    }

                },
                Err(_) => {
                    println!("Daemon is not running. Start it first using `startd (sd)`.");
                }
            }
            
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Killd => {
            println!("{}",kill_all().unwrap());
        },
        Commands::Startd { head } => {
            if let Err(e) = start_daemon(head) {
                        eprintln!("{}", e);
                        return;
            }
        },
        Commands::Display => {
            match tokio::runtime::Runtime::new() {
                Ok(rt) => {
                    if let Err(e) = rt.block_on(run_player_loop()) {
                        eprintln!("Error in player loop: {}", e);
                    }
                }
                Err(e) => eprintln!("Failed to create Tokio runtime: {}", e),
            }
        },
        Commands::GetPlayingTrackMeta => {
            println!("This is an internal daemon command and is not meant to be executed directly");
        }
        _=>{
            cmd_to_daemon(cli);
        },
    };
}