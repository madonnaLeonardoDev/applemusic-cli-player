use std::os::unix::net::UnixStream;
use std::io::{Write, Read};
use std::net::Shutdown;
use clap::{Parser, Subcommand};
use std::ffi::OsString;
use crate::paths::{DAEMON_SOCKET_PATH};
use crate::daemon::{kill_all, start_daemon};
mod daemon;
mod handle_client;
mod paths;
mod browser;
mod apple_music_navigation;
mod daemon_controls;

#[derive(Parser)]
struct Cli {

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(visible_alias = "sd")]
    Startd{
        #[arg(short = 'H', long)]
        head:bool
    },
    #[command(visible_alias = "kd")]
    Killd,

    #[command(external_subcommand)]
    External(Vec<OsString>)
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
        Commands::External(os_str) => {
            // Rejoin all remaining arguments into a single command string (e.g., "play track_name")
                    let args: Vec<String> = os_str
                        .iter()
                        .filter_map(|s| s.to_str().map(String::from))
                        .collect();
                    let full_command = args.join(" ");

                    // Connect to the daemon's UNIX socket
                    match UnixStream::connect(DAEMON_SOCKET_PATH.as_path()) {
                        Ok(mut stream) => {
                            if let Err(e) = stream.write_all(full_command.as_bytes()) {
                                println!("Error writing to stream: {}", e);
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
                        }
                        Err(_) => {
                            println!("Daemon is not running. Start it first using `stard (sd)`.");
                        }
                    }
        }
    }
}