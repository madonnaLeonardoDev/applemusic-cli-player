use std::env;
use std::os::unix::net::UnixStream;
use std::io::{Write, Read};
use std::net::Shutdown;

use crate::paths::{DAEMON_SOCKET_PATH};
use crate::daemon::{kill_all, start_daemon};
mod daemon;
mod handle_client;
mod paths;
mod browser;
mod apple_music_navigation;
mod daemon_controls;


fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if let Some(command) = args.get(0) {

        match command.as_str() {
                "start" => {

                    if let Err(e) = start_daemon() {
                        eprintln!("{}", e);
                        return;
                    }

                },
                "stop" => {
                    match kill_all() {
                        Ok(msg) => println!("{}", msg),
                        Err(e) => {println!("{}", e)}
                    }
                },
                _ =>  {
                    
                    // Rejoin all remaining arguments into a single command string (e.g., "play track_name")
                    let full_command = args.join("");

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
                            println!("Daemon is not running. Start it first using `daemon start`.");
                        }
                    }
            }
        }
    }
}