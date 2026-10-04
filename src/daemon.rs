use std::fs::{self};
use std::net::Shutdown;
use std::io::{Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use daemonize::Daemonize;
use crate::paths::{SOCKET_PATH, PID_PATH};
use crate::handle_client::{handle_client};
use crate::browser::{apple_music_auth, init_browser};


fn print_to_main(msg: &str, stream: &mut UnixStream) -> Result<(), String> {
    if let Err(e) = stream.write_all(msg.as_bytes()) {
        return Err(format!("Error: Could not write to stream: {}", e))
    }
    Ok(())
}


pub async  fn start_daemon() -> Result<(), String> {
    // 1. Check for a stale PID file
    if Path::new(PID_PATH.as_path()).exists() {
        if let Ok(pid_str) = fs::read_to_string(PID_PATH.as_path()) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                // Check if a process with this PID is actually running (using a signal 0 check or similar)
                // If it's dead, we can safely overwrite the stale files!
                println!("Found existing PID file. Checking if process {} is alive...", pid);
                
                // On Unix, sending signal 0 checks if the process exists without harming it
                let is_alive = unsafe { libc::kill(pid as i32, 0) == 0 };
                
                if is_alive {
                    return Err(format!("Daemon is already running (PID {}). Aborting.", pid));
                    
                } else {
                    println!("Found stale PID file (process is dead). Cleaning up...");
                }
            }
        }
        fs::remove_file(&*PID_PATH)
        .map_err(|e| format!("Could not remove file at: {}, {}", &PID_PATH.display(), e))?;
    }

    // 2. Clean up stale socket file if it exists
    if Path::new(SOCKET_PATH.as_path()).exists() {
    fs::remove_file(SOCKET_PATH.as_path())
    .map_err(|e| format!("Could not remove .sock file at: {}, {}", &SOCKET_PATH.display(), e))?;
    }

    // 3. Write new PID and bind socket...

    let stdout = fs::File::create("/tmp/applemusic_daemon.out")
        .map_err(|e| format!("Failed to create stdout log: {}", e))?;
    let stderr = fs::File::create("/tmp/applemusic_daemon.err")
        .map_err(|e| format!("Failed to create stderr log: {}", e))?;


    let daemonize = Daemonize::new()
        .pid_file(&*PID_PATH)
        .working_directory("/tmp")
        .stdout(stdout)
        .stderr(stderr);

    println!("Starting applemusic-daemon...");

    match daemonize.start() {
        Ok(_) => {
            let pid = std::process::id();
            println!("Daemon started successfully (PID {}).", pid);
        },
        Err(e) => {
            return Err(format!("Error in starting daemon: {}", e))
            
        }
    }
    
    if let Err(e) = apple_music_auth().await {
                    println!("{}", e);
                    return Err("Error, cant start apple music auth".to_string());
    }
    let state = init_browser(true).await?;

    let listener = UnixListener::bind(&*SOCKET_PATH)
    .map_err(|e| format!("Error in binding socket lisener: {}", e))?;
    

    // ... listener loop ...
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let response = match handle_client(&stream, &state) {
                    Ok(msg ) => {
                        msg
                    },
                    Err(e) => {
                        format!("[ERROR] {}", e)
                    }
                };
                
                if let Err(e) = print_to_main(&response, &mut stream) {
                        eprintln!("{}", e);
                        continue;
                }

                if let Err(e) = stream.shutdown(Shutdown::Write) {
                eprintln!("Error can't shutdown stream: {}", e);
                continue;
                }
            },
            Err(e) => {
                eprint!("Connection failed: {}", e)
            }
        }
    }
    Ok(())
}


pub fn kill_daemon() -> Result<String, String> {

    if !Path::new(&*PID_PATH).exists() {
        return Ok("Daemon is not running (no PID file found).".to_string());
    }

    let pid = 
    fs::read_to_string(&*PID_PATH)
    .map_err(|e| format!("Could not read pid at: {}, {}", &PID_PATH.display(), e))?
    .trim().parse::<i32>()
    .map_err(|e| format!("Could not parse pid, {}", e))?;

    unsafe {
        libc::kill(pid, libc::SIGTERM);
    };

    fs::remove_file(&*PID_PATH)
    .map_err(|e| format!("Could not remove PID file at: {}, {}",&PID_PATH.display(), e))?;

    fs::remove_file(&*SOCKET_PATH)
    .map_err(|e| format!("Could not remove socket file at: {}, {}", &SOCKET_PATH.display(), e))?;

    Ok(format!("Daemon killed successfully PID: {}", pid))
}