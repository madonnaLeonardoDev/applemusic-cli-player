use std::vec;
use crate::daemon_controls::format_time;
use thirtyfour::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Track {
    pub id: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_ms: Option<u32>
}

impl Track {
    pub fn pretty_display(&self) -> String {
    let duration = if self.duration_ms.is_some() {
        format_time(self.duration_ms.unwrap())
    } else {
        "Unknown Duration".to_string()
    };
    


    format!(
            "{} - (id: {})\n{} | {} | ({})",
            self.title.clone().unwrap_or_else(|| "Unknown Title".to_string()),
            self.id.clone().unwrap_or_else(|| "Unknown ID".to_string()),
            self.artist.clone().unwrap_or_else(|| "Unknown Artist".to_string()),
            self.album.clone().unwrap_or_else(|| "Unknown Album".to_string()),
            duration

        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Playlist {
    pub id: Option<String>,
    pub name: Option<String>,
    pub desc: Option<String>
}

impl Playlist {
    pub fn pretty_display(&self) -> String {
        format!(
            "{} - (id: {})\n{}",
            self.name.clone().unwrap_or_else(|| "Unknown Name".to_string()),
            self.id.clone().unwrap_or_else(|| "Unknown ID".to_string()),
            self.desc.clone().unwrap_or_default()
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Album {
    pub id: Option<String>,
    pub name: Option<String>,
    pub artist: Option<String>,
    pub track_count: Option<u32>,
}

impl Album {
    pub fn pretty_display(&self) -> String {
        format!(
            "{}(id: {})\n{} | Tracks: {}",
            self.name.clone().unwrap_or_else(|| "Unknown Name".to_string()),
            self.id.clone().unwrap_or_else(|| "Unknown ID".to_string()),
            self.artist.clone().unwrap_or_else(|| "Unknown Artist".to_string()),
            self.track_count.clone().unwrap_or_else(|| 0)
        )
    }
}

pub enum ItemType {
    Song,
    Playlist,
    Album
}

impl ItemType {
    fn as_str(&self) -> &'static str {
        match self {
            ItemType::Album => "albums",
            ItemType::Playlist => "playlists",
            ItemType::Song => "songs"
        }
    }
}

// Added internal tagging to match JS output { type: "Track", ... }
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SearchResult {
    Track(Track),
    Album(Album),
    Playlist(Playlist)
}

pub async fn search(driver: &WebDriver, query: String, item_type: &ItemType, is_library: bool) -> Result<SearchResult, String> {
    let script: String = if is_library {
        format!(
            r#"
            return (async () => {{
                try {{
                    const mk = MusicKit.getInstance();
                    if (!mk || !mk.isAuthorized) {{
                        await mk?.authorize();
                    }}

                    const searchResponse = await mk.api.music(`v1/me/library/search`, {{
                        term: "{}",
                        types: "library-{}",
                        limit: 1
                    }});

                    // Key in library responses is 'library-songs', 'library-albums', etc.
                    const data = searchResponse?.data?.results?.["library-{}"]?.data?.[0];
                    if (!data) return null;

                    const attr = data?.attributes || {{}};

                    if ("{}" === "songs") {{
                        return {{
                            type: "Track",
                            id: data.id || null,
                            title: attr.name || null,
                            artist: attr.artistName || null,
                            album: attr.albumName || null,
                            duration_ms: attr.durationInMillis || null
                        }};
                    }} else if ("{}" === "albums") {{
                        return {{
                            type: "Album",
                            id: data.id || null,
                            name: attr.name || null,
                            artist: attr.artistName || null,
                            track_count: attr.trackCount || null
                        }};
                    }} else if ("{}" === "playlists") {{
                        return {{
                            type: "Playlist",
                            id: data.id || null,
                            name: attr.name || null,
                            desc: attr.description?.standard || null
                        }};
                    }}

                    return null;
                }} catch (err) {{
                    console.error("Library Search Error:", err);
                    return null;
                }}
            }})();
            "#,
            query,
            item_type.as_str(),
            item_type.as_str(), // Fixed key prefix for library search
            item_type.as_str(),
            item_type.as_str(),
            item_type.as_str(),
        )
    } else {
        format!(
            r#"
            return (async () => {{
                try {{
                    const mk = MusicKit.getInstance();
                    if (!mk || !mk.isAuthorized) {{
                        await mk?.authorize();
                    }}

                    const storefront = mk.storefrontId || mk.storefront || 'us';

                    const searchResponse = await mk.api.music(`v1/catalog/${{storefront}}/search`, {{
                        term: "{}",
                        types: "{}",
                        limit: 1
                    }});

                    const data = searchResponse?.data?.results?.{}?.data?.[0];
                    if (!data) return null;

                    const attr = data?.attributes || {{}};

                    if ("{}" === "songs") {{
                        return {{
                            type: "Track",
                            id: data.id || null,
                            title: attr.name || null,
                            artist: attr.artistName || null,
                            album: attr.albumName || null,
                            duration_ms: attr.durationInMillis || null
                        }};
                    }} else if ("{}" === "albums") {{
                        return {{
                            type: "Album",
                            id: data.id || null,
                            name: attr.name || null,
                            artist: attr.artistName || null,
                            track_count: attr.trackCount || null
                        }};
                    }} else if ("{}" === "playlists") {{
                        return {{
                            type: "Playlist",
                            id: data.id || null,
                            name: attr.name || null,
                            desc: attr.description?.standard || null
                        }};
                    }}

                    return null;
                }} catch (err) {{
                    console.error("Catalog Search Error:", err);
                    return null;
                }}
            }})();
            "#,
            query,
            item_type.as_str(),
            item_type.as_str(),
            item_type.as_str(),
            item_type.as_str(),
            item_type.as_str(),
        )
    };

    let res = driver.execute(script, vec![]).await
        .map_err(|e| e.to_string())?;

    let search_result: Option<SearchResult> = serde_json::from_value(res.json().clone())
        .map_err(|e| format!("Deserialization error: {}", e))?;

    search_result.ok_or_else(|| "No Matches Found".to_string())
}

pub async fn play_next(driver: &WebDriver, item_type: &ItemType, id: &str) -> Result<String, String> {
    let script = format!(r#"
    return (async () => {{
        const mk = MusicKit.getInstance();
        if (!mk || !mk.isAuthorized ) {{
        await mk?.authorize();
    }}
    try {{ 
    await mk.playNext({{ {}: '{}'}});
    return true;
    }}
    catch (e) {{
        return false;
    }}
}})()
"#, item_type.as_str(), id);

let res = driver.execute(script, vec![]).await
    .map_err(|e| e.to_string())?;


let is_success: bool = serde_json::from_value(res.json().clone())
    .map_err(|e| e.to_string())?;

if is_success {
    return Ok(id.to_string())
}

Err("Could Not play next, maybe wrong id".to_string())
    
}
pub async fn play(driver: &WebDriver) -> Result<(), String> {
    let script = "MusicKit.getInstance().play();";
    driver.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
}
pub async fn pause(driver: &WebDriver) -> Result<(), String> {
    let script = "MusicKit.getInstance().pause();";
    driver.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
}
pub async fn next_track(driver: &WebDriver) -> Result<(), String> {
    let script = "MusicKit.getInstance().skipToNextItem();";
    driver.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
}
pub async fn previous_track(driver: &WebDriver) -> Result<(), String> {
    let script = "MusicKit.getInstance().skipToPreviousItem();";
    driver.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn shuffle_toggle(driver: &WebDriver) -> Result<i64, String> {
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
    Ok(num_mode)
}

pub async fn list_library_playlists(driver: &WebDriver) -> Result<Vec<Playlist>, String> {
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
                    name: p.attributes?.name || null,
                    desc: p.attributes?.description?.standard || null,
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
    serde_json::from_value::<Vec<Playlist>>(json_val.clone()).map_err(|e| {
        format!(
            "Deserialization failed for JSON `{}`: {}",
            json_val, e
        )
    })
}

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


pub async fn clear_queue(driver: &WebDriver) -> Result<(), String> {
    let script = r#"
    MusicKit.getInstance().clearQueue()
    "#;

    driver.execute(script, vec![]).await.map_err(|e| e.to_string())?;
    
    Ok(())
}

pub async fn list_queue(driver: &WebDriver) -> Result<Vec<Track>, String> {
 let script = r#"
    return (async () => {
    let res = MusicKit.getInstance().queue._queueItems || [];
    return res.map(e => ({
        id: e.item.id,
        title: e.item.attributes.name,
        artist: e.item.attributes.artistName,
        album: e.item.attributes.albumName,
        duration_ms: e.item.attributes.durationInMillis
    }));
})();
 "#;

let res = driver.execute(script, vec![]).await
    .map_err(|e| e.to_string())?;

let parsed_res: Vec<Track> = serde_json::from_value(res.json().clone())
    .map_err(|e| e.to_string())?;



 Ok(parsed_res)
} 

pub async fn volume(driver: &WebDriver, val: &Option<u32>) -> Result<i32, String> {
    let script =
        if val.is_none() {
                r#"
                return (async () => {
                    return MusicKit.getInstance().volume;
        })();
                "#.to_string()
        } else {
            format!(
                r#"
                return (async () => {{
                    MusicKit.getInstance().volume = {};
                    return MusicKit.getInstance().volume;
            }})();
                "#, val.unwrap() as f64 / 100.0
            )
        };

        let res = driver.execute(script, vec![]).await
            .map_err(|e| e.to_string())?;

        let parsed_res: f64 = serde_json::from_value(res.json().clone())
            .map_err(|e| e.to_string())?;
    Ok((parsed_res * 100.0).round() as i32)
}