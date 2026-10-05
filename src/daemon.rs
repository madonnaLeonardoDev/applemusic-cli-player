use std::fs::{self};
use tokio::io::{AsyncWriteExt};
use std::path::Path;
use daemonize::Daemonize;
use crate::paths::{DAEMON_SOCKET_PATH, DAEMON_PID_PATH, GECKO_PID_PATH};
use crate::handle_client::{handle_client};
use crate::browser::{apple_music_auth, init_browser};


pub fn start_daemon() -> Result<(), String> {
//CHECK AND CLEANUP (SOCK + PID)    

    if Path::new(DAEMON_PID_PATH.as_path()).exists() {
        if let Ok(pid_str) = fs::read_to_string(DAEMON_PID_PATH.as_path()) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                println!("Found existing PID file. Checking if process {} is alive...", pid);

                let is_alive = unsafe { libc::kill(pid as i32, 0) == 0 };
                
                if is_alive {
                    return Err(format!("Daemon is already running (PID {}). Aborting.", pid));
                    
                } else {
                    println!("Found stale PID file (process is dead). Cleaning up...");
                }
            }
        }
        fs::remove_file(&*DAEMON_PID_PATH)
        .map_err(|e| format!("Could not remove file at: {}, {}", &DAEMON_PID_PATH.display(), e))?;
    }

    if Path::new(DAEMON_SOCKET_PATH.as_path()).exists() {
    fs::remove_file(DAEMON_SOCKET_PATH.as_path())
    .map_err(|e| format!("Could not remove .sock file at: {}, {}", &DAEMON_SOCKET_PATH.display(), e))?;
    }

//SETUP deamonize

    let stdout = fs::File::create("/tmp/applemusic_daemon.out")
        .map_err(|e| format!("Failed to create stdout log: {}", e))?;
    let stderr = fs::File::create("/tmp/applemusic_daemon.err")
        .map_err(|e| format!("Failed to create stderr log: {}", e))?;


    let daemonize = Daemonize::new()
        .pid_file(&*DAEMON_PID_PATH)
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
    
// async tokio runtime
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to boot async runtime: {}", e))?;

    // Returning this block's Result directly fixes the "unused result" warning
    // and correctly bubbles up startup/initialization errors.
    rt.block_on(async {
        // Auth check
        if let Err(e) = apple_music_auth().await {
            eprintln!("Auth error: {}", e);
            return Err("Error, cant start apple music auth".to_string());
        }
        let state = init_browser(true).await?;

        // Bind Tokio async Unix socket listener
        let listener = tokio::net::UnixListener::bind(&*DAEMON_SOCKET_PATH)
            .map_err(|e| format!("Error in binding socket listener: {}", e))?;

        // 4. Async accept loop
        loop {
            match listener.accept().await {
                Ok((mut stream, _)) => {
                    let response = match handle_client(&mut stream, &state.driver).await {
                        Ok(msg) => msg,
                        Err(e) => e.to_string(),
                    };

                    // Write response asynchronously over the socket
                    if let Err(e) = stream.write_all(response.as_bytes()).await {
                        eprintln!("Error writing response: {}", e);
                        continue;
                    }

                    // Shutdown stream write-half asynchronously
                    if let Err(e) = stream.shutdown().await {
                        eprintln!("Error shutting down stream: {}", e);
                        continue;
                    }
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }
    })
}


pub fn kill_all() -> Result<String, String> {
    // Gracefully kill daemon if PID file exists
    let _ = kill_pid(&*DAEMON_PID_PATH);

    // Gracefully kill geckodriver if PID file exists
    let _ = kill_pid(&*GECKO_PID_PATH);

    // Safely remove Socket file only if it was actually created
    if Path::new(&*DAEMON_SOCKET_PATH).exists() {
        fs::remove_file(&*DAEMON_SOCKET_PATH)
            .map_err(|e| format!("Could not remove socket file at: {}, {}", &DAEMON_SOCKET_PATH.display(), e))?;
    }

    Ok("All Processes killed successfully".to_string())
}

pub fn kill_pid(pid_path: &Path) -> Result<String, String> {
    if !Path::new(pid_path).exists() {
        // Return Ok instead of Err so it doesn't abort kill_all if a process wasn't started
        return Ok(format!("{} is not running", pid_path.display()));
    }

    let pid_str = fs::read_to_string(pid_path)
        .map_err(|e| format!("Could not read pid at: {}, {}", pid_path.display(), e))?;
        
    let pid = pid_str.trim().parse::<i32>()
        .map_err(|e| format!("Could not parse pid, {}", e))?;

    // Send SIGTERM to the process
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    };

    // Safely remove PID file if it exists
    if Path::new(pid_path).exists() {
        fs::remove_file(pid_path)
            .map_err(|e| format!("Could not remove PID file at: {}, {}", pid_path.display(), e))?;
    }

    Ok(format!("PID: {} killed successfully", pid_path.display()))
}