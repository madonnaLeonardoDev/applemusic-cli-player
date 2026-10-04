use thirtyfour::prelude::*;
use std::time::Duration;
use tokio::time::sleep;
use std::process::{Command, Child, Stdio};
use crate::paths::BROWSER_PROFILE_PATH;

// We need to keep track of both the browser session and the driver server
pub struct BrowserState {
    pub driver: WebDriver,
    pub geckodriver_process: Child,
}

pub async fn init_browser(is_headless: bool) -> Result<BrowserState, String> {
    // 1. Spawn geckodriver in the background so you don't have to run it manually
    let geckodriver_process = Command::new("geckodriver")
        .arg("--port")
        .arg("4444")
        .stdout(Stdio::null()) // Mute standard output
        .stderr(Stdio::null()) // Mute error output (the javascript warnings)
        .spawn()
        .map_err(|e| format!("Failed to start geckodriver: {}", e))?;

    // Give geckodriver half a second to bind to port 4444
    sleep(Duration::from_millis(500)).await;

    // 2. Configure Firefox capabilities
    let mut caps = DesiredCapabilities::firefox();
    
    if is_headless {
        caps.set_headless().map_err(|e| e.to_string())?;
    }
    
    // Convert BROWSER_PROFILE_PATH to a string and pass it to Firefox arguments
    let profile_path = BROWSER_PROFILE_PATH.to_str().unwrap_or("/tmp/apple-music-profile");
    caps.add_arg("-profile").map_err(|e| e.to_string())?;
    caps.add_arg(profile_path).map_err(|e| e.to_string())?;

    // 3. Connect thirtyfour to the running geckodriver instance
    let driver = WebDriver::new("http://localhost:4444", caps)
        .await
        .map_err(|e| e.to_string())?;

    Ok(BrowserState { driver, geckodriver_process })
}

pub async fn open_apple_music(driver: &WebDriver) -> Result<(), String> {
    // thirtyfour's goto() implicitly waits for the page load to finish
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