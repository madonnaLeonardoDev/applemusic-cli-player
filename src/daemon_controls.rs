use thirtyfour::WebDriver;
use crate::apple_music_navigation::*;

fn format_time(time_ms: u64) -> String {
    let tot_secs = time_ms / 1000;
    let seconds = tot_secs % 60;
    let minutes = tot_secs / 60;
    let hours = tot_secs / 3600;

    if hours > 0 {
        
        return format!("{:02}:{:02}",minutes, seconds);
        
    } else {
        return format!("{}:{:02}:{:02}", hours, minutes, seconds);
    }
}


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

pub async fn cmd_current_track(driver:&WebDriver) -> Result<String, String> {

    if let Some(track) = get_current_track(driver).await? {
        let title:String = if track.title.is_none(){
            "Unkown Title".to_string()
        } else {
            track.title.unwrap()
        };
        let artist: String = if track.artist.is_none() {
            "Unkown Artist".to_string()
        } else {
            track.artist.unwrap()
        };


        let album: String = if track.album.is_none(){
            "Unkown Album".to_string()
        } else {
            track.album.unwrap()
        };

        let duration: String = if track.duration_ms.is_none() {
            "Unkown Duration".to_string()
        } else {
            format_time(track.duration_ms.unwrap())
        };
        return Ok(format!(
            "Track Playing:\n\r{}\n\r{} | {} ({})",
            title, artist, album, duration 
        ))
    }

    Ok("No Track playing".to_string())
    
}

pub async fn cmd_list_playlists(driver:&WebDriver) -> Result<String, String> {
    match list_library_playlists(driver).await {
    Ok(pl) => {
        let result: String = pl
        .iter()
        .map(|pl| format!("-{}\n\r{}", pl.name, pl.desc))
        .collect();
        Ok(result)
    },
    Err(e) => return Err(e)
    }
}
