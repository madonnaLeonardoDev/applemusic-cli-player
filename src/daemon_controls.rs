use thirtyfour::WebDriver;

use crate::apple_music_navigation::{play, pause, next_track, previous_track, play_item, list_library_playlists, search_and_play};

pub async fn cmd_play_pause(driver:&WebDriver) -> Result<String, String> {
    let script = r#"
        const mk = MusicKit.getInstance();
        return mk ? (mk.isPlaying || mk.playbackState === 2) : false;
    "#;

    let eval_result = driver.execute(script, vec![])
        .await
        .map_err(|e| e.to_string())?;

    // Extracts the boolean from the returned JSON payload
    let is_playing = eval_result.json().as_bool().unwrap_or(false);

    if is_playing {
        if let Err(e) = pause(driver).await {
            return Err(e.to_string());
        };

        return Ok("Pause".to_string())
    } else {
        if let Err(e) = play(driver).await {
            return Err(e.to_string());
        };

        return Ok("Play".to_string())
    }
}

pub async fn cmd_next(driver:&WebDriver) -> Result<String, String> {
    if let Err(e) = next_track(driver).await {
        return Err(e.to_string());
    }

    Ok("Next Track".to_string())
}

pub async fn cmd_prev(driver:&WebDriver) -> Result<String, String> {
    if let Err(e) = previous_track(driver).await {
        return Err(e.to_string());
    }

    Ok("Previous Track".to_string())
}