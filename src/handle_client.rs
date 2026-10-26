
use thirtyfour::WebDriver;
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, BufReader};
use crate::apple_music_navigation::ItemType;
use crate::{daemon_controls::*};
use crate::Commands;


pub async fn handle_client(stream: &mut UnixStream, driver: &WebDriver) -> Result<String, String> {
    let mut reader: BufReader<&mut UnixStream> = BufReader::new(stream);

    let mut len_buf = [0u8; 4];

    reader.read_exact(&mut len_buf).await
        .map_err(|e| format!("Failed reading length prefix: {}", e))?;

    let len = u32::from_le_bytes(len_buf) as usize;

    if len <= 0 {
        return Err("Invalid empty command".to_string());
    }

    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload).await
        .map_err(|e| format!("Failed reading payload: {}", e))?;

    let command: Commands = wincode::deserialize(&payload)
        .map_err(|e| format!("Failed Deserializing payload from bytes: {}", e))?;
    



    let result: Result<String, String> =  match command {
        Commands::Search { stype, query, next, insta_play, library } => {
            let search_type = match stype.to_lowercase().as_str() {
                "song" | "s" => ItemType::Song,
                "album" | "a" => ItemType::Album,
                "playlist" | "p" => ItemType::Playlist,
                _ => {
                   return Err("Invalid search type (stype) argument".to_string());
                }
            };
            let joined_query = query.replace("-", " ");
            Ok(cmd_search(driver, joined_query, &search_type, library, next, insta_play).await?)
        },
        Commands::Current => {
            Ok(cmd_current_track(driver).await?)
        },
        Commands::LsPlaylists {  } => {
            Ok(cmd_list_playlists(driver).await?)
        },
        Commands::Playid { stype, id, next } => {
            let search_type = match stype.to_lowercase().as_str() {
                "song" | "s" => ItemType::Song,
                "album" | "a" => ItemType::Album,
                "playlist" | "p" => ItemType::Playlist,
                _ => {
                    return Err("Invalid search type (stype) argument".to_string());
                }
            };

            Ok(cmd_play_by_id(driver, id, search_type, next).await?)
        },
        Commands::PlayPause => {
            Ok(cmd_play_pause(driver).await?)
        },
        Commands::Next  => {
            Ok(cmd_next(driver).await?)
        },
        Commands::Prev  => {
            Ok(cmd_prev(driver).await?)
        },
        Commands::Shuffle => {
            Ok(cmd_toggle_shuffle(driver).await?)
        },
        Commands::ClearQueue => {
            Ok(cmd_clear_queue(driver).await?)
        },
        Commands::LsQueue => {
            Ok(cmd_list_queue(driver).await?)
        },
        Commands::Volume { args } => {
            Ok(cmd_volume(driver, args).await?)
        },
        _=> {
            Err("Invalid Command".to_string())
        }
    };

    if let Err(e) = result {
        return Ok(e.to_string());
    }
    Ok(result.unwrap())
 
} 