use thirtyfour::prelude::*;
use std::{path::Path, time::Duration, fs};
use tokio::time::sleep;
use std::process::{Command, Child, Stdio};
use crate::paths::{BROWSER_PROFILE_PATH, GECKO_PID_PATH, BROWSER_PID_PATH};
pub struct BrowserState {
    pub driver: WebDriver,
    pub geckodriver_process: Child,
}

pub async fn init_browser(is_headless: bool) -> Result<BrowserState, String> {
    // 3. CLEAN UP STALE FIREFOX PROFILE LOCK
    let lock_file = BROWSER_PROFILE_PATH.join("/lock");
    if lock_file.exists() {
        if let Err(e) = fs::remove_file(&lock_file) {
            eprintln!("Could not remove stale profile lock: {}", e);
        } else {
            println!("Removed stale Firefox profile lock.");
        }
    }

    // 1. CLEAN UP STALE GECKO
    if Path::new(&*GECKO_PID_PATH).exists() {
        if let Ok(pid_str) = fs::read_to_string(&*GECKO_PID_PATH) {
            match pid_str.trim().parse::<i32>() {
                Ok(pid) => {
                    unsafe { libc::kill(pid, libc::SIGTERM); }
                }
                Err(_) => {
                    eprintln!("Failed Parsing GECKO_DRIVER pid");
                }
            }
        }

        if let Err(e) = fs::remove_file(&*GECKO_PID_PATH) {
            eprintln!("Could not remove stale GECKO_DRIVER pid: {}", e);
        }
    }

    // 2. CLEAN UP STALE BROWSER (if tracked)
    if Path::new(&*BROWSER_PID_PATH).exists() {
        if let Ok(pid_str) = fs::read_to_string(&*BROWSER_PID_PATH) {
            match pid_str.trim().parse::<i32>() {
                Ok(pid) => {
                    unsafe { libc::kill(pid, libc::SIGTERM); }
                }
                Err(_) => {
                    eprintln!("Failed Parsing BROWSER pid");
                }
            }
        }

        if let Err(e) = fs::remove_file(&*BROWSER_PID_PATH) {
            eprintln!("Could not remove stale BROWSER pid: {}", e);
        }
    }

    // 3. GET DYNAMIC PORT
    let temp_listener = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Failed to find a free port: {}", e))?;
    
    let dynamic_port = temp_listener.local_addr()
        .map_err(|e| format!("Failed to read local port address: {}", e))?
        .port();
    drop(temp_listener);

    // 4. Spawn geckodriver in the background
    let geckodriver_process = Command::new("geckodriver")
        .arg("--port")
        .arg(dynamic_port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start geckodriver: {}", e))?;

    // CRITICAL FIX: Write the new geckodriver PID to disk so cleanup can find it next time!
    let child_pid = geckodriver_process.id();
    if let Err(e) = fs::write(&*GECKO_PID_PATH, child_pid.to_string()) {
        eprintln!("Failed to write gecko PID file: {}", e);
    }

    sleep(Duration::from_millis(500)).await;

    // 5. Configure Firefox capabilities with persistent profile path
    let mut caps = DesiredCapabilities::firefox();
    
    if is_headless {
        caps.set_headless().map_err(|e| e.to_string())?;
    }
    
    
    // Convert BROWSER_PROFILE_PATH to a string and pass it to Firefox arguments
    if let Some(profile_path) = BROWSER_PROFILE_PATH.to_str() {
    // 1. Resolve to a absolute path (Geckodriver crashes without this)
    let abs_path = std::fs::canonicalize(profile_path)
        .map_err(|e| format!("Failed to canonicalize path: {}", e))?
        .to_string_lossy()
        .to_string();

    // 2. Pass --profile (DOUBLE DASH) and the absolute path
    caps.add_arg("-profile").map_err(|e| e.to_string())?;
    caps.add_arg(&abs_path).map_err(|e| e.to_string())?;
    } else {
        return Err("Profile path returned None".to_string());
    }
    // 6. Connect thirtyfour to the running geckodriver instance
    let driver = WebDriver::new(format!("http://127.0.0.1:{}", dynamic_port), caps)
        .await
        .map_err(|e| format!("Error connecting WebDriver to port {}: {}", dynamic_port, e))?;

    Ok(BrowserState { driver, geckodriver_process })
}

pub async fn open_apple_music(driver: &WebDriver) -> Result<(), String> {
    driver.goto("https://music.apple.com")
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}








pub async fn is_logged_in(driver: &WebDriver) -> Result<bool, String> {
    // Notice the "return" keyword! thirtyfour executes scripts differently.
    // It must return a value back to Rust explicitly.
    let script = "return document.querySelector('.account-menu > amp-contextual-menu-button:nth-child(1) > button:nth-child(1)') !== null;";

    let eval_result = driver.execute(script, vec![])
        .await
        .map_err(|e| e.to_string())?;

    // Extract the boolean from the returned JSON value
    let is_logged_in = eval_result.json().as_bool().unwrap_or(false);

    Ok(is_logged_in)
}

pub async fn apple_music_auth() -> Result<(), String> {
    // state contains both the active browser driver and the geckodriver process handle
    let mut state = init_browser(false).await?;
    
    open_apple_music(&state.driver).await?;

    if is_logged_in(&state.driver).await? {
        println!("User already logged in using saved session");
    } else {
        println!("Please log in manually thru the browser window...");

        while !is_logged_in(&state.driver).await? {
            sleep(Duration::from_secs(3)).await;
        }

        println!("Logged in succesfully!");
    }

    // 1. Close the browser window and flush session data
    state.driver.quit().await.map_err(|e| e.to_string())?;
    
    // 2. Kill the background geckodriver server process cleanly
    state.geckodriver_process.kill().map_err(|e| e.to_string())?;

    Ok(())
}