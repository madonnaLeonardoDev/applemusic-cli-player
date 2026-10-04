use thirtyfour::prelude::*;
use serde::{Serialize, Deserialize};

/// Resume playback
pub async fn play(driver: &WebDriver) -> Result<(), String> {
    let script = "return MusicKit.getInstance().play();";
    driver.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Pause playback
pub async fn pause(driver: &WebDriver) -> Result<(), String> {
    let script = "return MusicKit.getInstance().pause();";
    driver.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Skip to next track
pub async fn next_track(driver: &WebDriver) -> Result<(), String> {
    let script = "return MusicKit.getInstance().skipToNextItem();";
    driver.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Skip to previous track
pub async fn previous_track(driver: &WebDriver) -> Result<(), String> {
    let script = "return MusicKit.getInstance().skipToPreviousItem();";
    driver.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Play a specific song or playlist by ID (e.g. song ID "1440854481" or playlist "pl.xyz")
pub async fn play_item(driver: &WebDriver, item_id: &str, is_playlist: bool) -> Result<(), String> {
    let queue_type = if is_playlist { "playlist" } else { "song" };
    let script = format!(
        "return MusicKit.getInstance().setQueue({{ {}: '{}' }}).then(() => MusicKit.getInstance().play());",
        queue_type, item_id
    );
    
    driver.execute(&script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistInfo {
    pub id: String,
    pub name: String,
    pub track_count: usize,
}

pub async fn list_library_playlists(driver: &WebDriver) -> Result<Vec<PlaylistInfo>, String> {
    let script = r#"
        const mk = MusicKit.getInstance();
        return mk.api.music('/v1/me/library/playlists', { limit: 100 })
            .then(res => {
                if (!res || !res.data || !res.data.data) return [];
                return res.data.data.map(item => ({
                    id: item.id,
                    name: item.attributes.name || 'Untitled Playlist',
                    track_count: item.attributes.trackCount || 0
                }));
            });
    "#;

    let res = driver.execute(script, vec![])
        .await
        .map_err(|e| format!("Failed to fetch playlists: {}", e))?;

    let playlists: Vec<PlaylistInfo> = serde_json::from_value(res.json().clone())
        .map_err(|e| format!("Failed to parse playlist data: {}", e))?;

    Ok(playlists)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchHit {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
}

pub async fn search_and_play(
    driver: &WebDriver, 
    query: &str, 
    is_playlist: bool
) -> Result<SearchHit, String> {
    let item_type = if is_playlist { "playlists" } else { "songs" };
    let queue_key = if is_playlist { "playlist" } else { "song" };

    // Escape single quotes in user input to prevent JS injection breaking the string
    let safe_query = query.replace('\'', "\\'");

    let script = format!(
        r#"
        const mk = MusicKit.getInstance();
        const itemType = '{item_type}';
        const queueKey = '{queue_key}';
        
        return mk.api.music('/v1/catalog/' + mk.storefrontId + '/search', {{
            term: '{safe_query}',
            types: itemType,
            limit: 1
        }}).then(res => {{
            const results = res.data.results[itemType];
            if (!results || !results.data || results.data.length === 0) {{
                throw new Error('No ' + itemType + ' found matching query: {safe_query}');
            }}
            
            const topHit = results.data[0];
            const hitId = topHit.id;
            const hitTitle = topHit.attributes.name;
            const hitArtist = topHit.attributes.artistName || null;

            return mk.setQueue({{ [queueKey]: hitId }})
                .then(() => mk.play())
                .then(() => ({{
                    id: hitId,
                    title: hitTitle,
                    artist: hitArtist
                }}));
        }});
        "#,
        item_type = item_type,
        queue_key = queue_key,
        safe_query = safe_query
    );

    let res = driver.execute(&script, vec![])
        .await
        .map_err(|e| format!("Search & play failed: {}", e))?;

    let hit: SearchHit = serde_json::from_value(res.json().clone())
        .map_err(|e| format!("Failed to parse search hit: {}", e))?;

    Ok(hit)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentTrack {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u64,
    pub artwork_url: Option<String>,
}

/// Returns metadata for the currently playing item, or `Ok(None)` if nothing is queued/playing.
pub async fn get_current_track(driver: &WebDriver) -> Result<Option<CurrentTrack>, String> {
    let script = r#"
        const mk = MusicKit.getInstance();
        if (!mk || !mk.nowPlayingItem) return null;

        const item = mk.nowPlayingItem;
        const attr = item.attributes || {};

        let artworkUrl = null;
        if (attr.artwork && attr.artwork.url) {
            // Replace dimensions placeholders with 300x300
            artworkUrl = attr.artwork.url.replace('{w}', '300').replace('{h}', '300');
        }

        return {
            id: item.id || '',
            title: attr.name || item.title || 'Unknown Title',
            artist: attr.artistName || item.artistName || 'Unknown Artist',
            album: attr.albumName || item.albumName || 'Unknown Album',
            duration_ms: attr.durationInMillis || 0,
            artwork_url: artworkUrl
        };
    "#;

    let eval_result = driver.execute(script, vec![])
        .await
        .map_err(|e| e.to_string())?;

    let value = eval_result.json();
    
    // If no song is loaded in the player, return None
    if value.is_null() {
        return Ok(None);
    }

    let track: CurrentTrack = serde_json::from_value(value.clone())
        .map_err(|e| format!("Failed to parse track metadata: {}", e))?;

    Ok(Some(track))
}