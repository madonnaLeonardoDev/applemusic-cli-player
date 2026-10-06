use thirtyfour::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Track {
    pub id: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_ms: Option<u64>
}
#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistInfo {
    pub id: String,
    pub name: String,
    pub desc: String
}

//Should work not tested
///get id
pub async fn get_id(driver: &WebDriver, query: String, query_type: String) -> Result<String, String> {
    let script = format!(r#"
    (async () => {{
    const mk = MusicKit.getInstance();
    const results = await mk.api.music(`v1/me/storefront`, {{}}); // Optional: get storefront dynamically
    const searchResponse = await mk.api.music('v1/catalog/us/search', {{
        term: '{}',
        types: '{}',
        limit: 1
}});
    const firstSong = searchResponse?.data?.results?.songs?.data?.[0];
    const firstId = firstSong ? firstSong.id : null;

    console.log("First result ID:", firstId);
}})(); "#, query, query_type);
    let res = driver.execute(script, vec![]).await
        .map_err(|e| e.to_string())?;

    let id_str: String = serde_json::from_value(res.json().clone())
        .map_err(|e| e.to_string())?;

    Ok(id_str)
}

//Should work not tested
/// Resume playback
pub async fn play(driver: &WebDriver) -> Result<(), String> {
    let script = "MusicKit.getInstance().play();";
    driver.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
}

//Should work not tested
/// Pause playback
pub async fn pause(driver: &WebDriver) -> Result<(), String> {
    let script = "MusicKit.getInstance().pause();";
    driver.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
}

//Should work not tested
/// Skip to next track
pub async fn next_track(driver: &WebDriver) -> Result<(), String> {
    let script = "MusicKit.getInstance().skipToNextItem();";
    driver.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
}


//Should work not tested
/// Skip to previous track
pub async fn previous_track(driver: &WebDriver) -> Result<(), String> {
    let script = "MusicKit.getInstance().skipToPreviousItem();";
    driver.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
}

//Should work not tested
pub async fn shuffle_toggle(driver: &WebDriver) -> Result<(), String> {
    let script = r#"
        return window.MusicKit.getInstance().shuffleMode;
    "#;

    let value = driver.execute(script, vec![]).await
        .map_err(|e| format!("Failed to execute script: {}", e))?;

    // Convert the serde_json::Value to a Rust boolean
    let is_shuffled: i64 = value.json().as_i64().unwrap_or(0);

    let num_mode = if is_shuffled == 1 {
        0
    } else {
        1
    };

    let script = format!{"MusicKit.getInstance().shuffleMode = {};", num_mode};
    driver.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
}

//WORKING TESTED
pub async fn list_library_playlists(driver: &WebDriver) -> Result<Vec<PlaylistInfo>, String> {
    let script = r#"
        return (async () => {
            try {
                const mk = MusicKit.getInstance();
                if (!mk || !mk.isAuthorized) {
                    await mk?.authorize();
                }

                const response = await mk.api.music('v1/me/library/playlists', { limit: 100 });
                const rawItems = response?.data?.data || [];

                // 2. Return the FLATTENED mapped array, NOT 'rawItems'
                return rawItems.map(p => ({
                    id: p.id,
                    name: p.attributes?.name || "Untitled Playlist",
                    desc: p.attributes?.description?.standard || "",
                }));
            } catch (err) {
                console.error("MusicKit error:", err);
                return [];
            }
        })();
    "#;

    let res = driver
        .execute(script, vec![])
        .await
        .map_err(|e| format!("WebDriver script execution failed: {}", e))?;

    let json_val = res.json();

    // 3. Explicit error reporting instead of silent fallback
    serde_json::from_value::<Vec<PlaylistInfo>>(json_val.clone()).map_err(|e| {
        format!(
            "Deserialization failed for JSON `{}`: {}",
            json_val, e
        )
    })
}

//Should work not tested
/// Returns metadata for the currently playing item, or `Ok(None)` if nothing is queued/playing.
pub async fn get_current_track(driver: &WebDriver) -> Result<Option<Track>, String> {
    let script = r#"
return (() => {
    let title = null;
    let artist = null;
    let album = null;
    let id = null;
    let duration = null;

    if ('mediaSession' in navigator && navigator.mediaSession.metadata) {
        title = navigator.mediaSession.metadata.title || title;
        artist = navigator.mediaSession.metadata.artist || artist;
        album = navigator.mediaSession.metadata.album || album;
    }

    const songLink = document.querySelector('footer a[href*="/song/"], [data-testid="player-controls"] a[href*="/song/"]');
    if (songLink) {
        const match = songLink.href.match(/\/song\/([^\/?#]+)/);
        if (match) {
            id = match[1];
        }
    }

    const audioEl = document.querySelector('audio');
    if (audioEl && !isNaN(audioEl.duration)) {
        duration = Math.round(audioEl.duration * 1000); // Converted to milliseconds
    }

    return { id, title, artist, album, duration };
})();
    "#;

    let res = driver.execute(script, vec![])
        .await
        .map_err(|e| e.to_string())?;

    let track_obj: Track = serde_json::from_value(res.json().clone())
        .map_err(|e| e.to_string())?;

    if track_obj.title.is_none() {
        return Ok(None)
    }
    Ok(Some(track_obj))
}