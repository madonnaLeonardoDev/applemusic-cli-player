use thirtyfour::{WebDriver};
use crate::apple_music_navigation::*;

fn format_time(time_ms: u32) -> String {
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

//TESTED WORKS
pub async fn cmd_search(driver:&WebDriver, query: String, search_type: &ItemType, is_library: bool, queue: bool, insta_play: bool) -> Result<String, String> {
    let search_res = search(driver, query, search_type,is_library).await?;

    let result_tuple:(String, String) = match search_res {
        SearchResult::Track(track) => {
            let id = if track.id.is_none(){
                return  Ok("No matches found".to_string());
                } else {
                    track.id.unwrap()
                };
            
            let time = if track.duration_ms.is_none(){
                "Unkown Duration".to_string()
            } else {
                format_time(track.duration_ms.unwrap())
            };
            
            (format!("{} - (id:{})\n{} | {} | {}", track.title.unwrap_or("Unkown title".to_string()),id ,track.artist.unwrap_or("Unkown Artist".to_string()), track.album.unwrap_or("Unkown Album".to_string()), time), id)
    },
    SearchResult::Playlist(playlist) => {
        let id = if playlist.id.is_none() {
            return Ok("No Matches found".to_string());
        } else {
            playlist.id.unwrap()
        };

        (format!("{} - (id:{})\n{}", playlist.name.unwrap_or("Unkown Name".to_string()), id, playlist.desc.unwrap_or("".to_string())), id)
    },
    SearchResult::Album(album) => {
        let id = if album.id.is_none() {
            return Ok("No Matches Found".to_string());
        } else {
            album.id.unwrap()
        };

        let track_count = if album.track_count.is_none() {
            "".to_string()
        } else {
            format!("{}",album.track_count.unwrap())
        };

        (format!("{} - (id:{})\n {} | Tracks:{}", album.name.unwrap_or("Unkown Name".to_string()), id, album.artist.unwrap_or("Unkown Artist".to_string()), track_count), id)

    }

    };
    if queue{
                play_next(driver, search_type, &result_tuple.1).await?;
            }
    if insta_play {
        play_next(driver, search_type, &result_tuple.1).await?;
        next_track(driver).await?
    }
    if queue && insta_play {
        return Err("Either Insta-Play (-p) or Play-Next (-n)".to_string());
    }
    Ok(result_tuple.0)
}

//TESTED WORKS
pub async fn cmd_play_by_id(driver:&WebDriver, id:String, item_type: ItemType, insta_play: bool) -> Result<String, String> {
    let id = play_next(driver, &item_type, &id).await
    .map_err(|e| e.to_string())?;

    if insta_play {
        next_track(driver).await?;
        return Ok(format!("Playing Now: {}",id));
    }
    Ok(format!("Playing Next: {}",id))
}

//TESTED WORKS
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

//TESTED WORKS
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

//TESTED WORKS (Unkown Duration maybe bug)

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


//TESTED WORKS
pub async fn cmd_list_playlists(driver:&WebDriver) -> Result<String, String> {
    match list_library_playlists(driver).await {
    Ok(pl) => {
        let result: String = pl
        .iter()
        .map(|pl| pl.pretty_display())
        .collect();
        Ok(result)
    },
    Err(e) => return Err(e)
    }
}

//
pub async fn cmd_clear_queue(driver:&WebDriver) -> Result<String, String> {
    clear_queue(driver).await?;

    Ok("Queue Cleared".to_string())
}

//TESTED WORKS
pub async fn cmd_toggle_shuffle(driver:&WebDriver) -> Result<String, String> {
    let mode = shuffle_toggle(driver).await
        .map_err(|e| e.to_string())?;

    if mode == 1{
        return Ok("Shuffling On".to_string());
    }
    Ok("Shuffling Off".to_string())
}