use thirtyfour::WebDriver;
use tokio::net::UnixStream;
use tokio::io::{AsyncBufReadExt,BufReader};

use crate::daemon_controls::*;


pub async fn handle_client(stream: &mut UnixStream, driver: &WebDriver) -> Result<String, String> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    let bytes_read = reader.read_line(&mut line).await
    .map_err(|e| format!("Could not read stream: {}", e))?;

    if bytes_read <= 0 {
        return Err("Invalid empty command".to_string());
    }

    let line_args: Vec<&str> = line.trim().split_whitespace().collect();

    let command = *line_args.get(0)
    .ok_or_else(||"Invalid empty command".to_string())?;



    let result: Option<String> =  match command {
        //CORE COMMANDS
        "next" => {
            None
        },
        "previous" => {
            None
        },
        "play_pause" => {
            None
        },
        //NAVIGATION COMMANDS

        "current" => {
            None
        },
        "chplaylist" => {
           let _ = check_args(&line_args, 1)?;
           None
        },
        "lsplaylist" => {
           Some(cmd_list_playlists(driver).await?)
        },
        "qnext" => {
            let _ = check_args(&line_args, 1)?;
            None
        },
        "playsong" => {
            let _ = check_args(&line_args, 1)?;
            None
        },
        _ => {
           None
        }
    };

    if result.is_none() {
        return Ok(format!("{} is not a command", command));
    }

    Ok(result.unwrap())

}

fn check_args(args_vec: &Vec<&str>, index: usize) -> Result<String, String>{
    
    if args_vec.get(index).is_none() {
        return Err("This command requires an argument".to_string());
    };
    Ok(args_vec[index].to_string())
}