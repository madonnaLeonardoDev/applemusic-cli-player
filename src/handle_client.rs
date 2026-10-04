use std::{io::{BufRead, BufReader}, os::unix::net::UnixStream};

use crate::browser::BrowserState;


pub fn handle_client(stream: &UnixStream, state: &BrowserState) -> Result<String, String> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    let bytes_read = reader.read_line(&mut line)
    .map_err(|e| format!("Could not read stream: {}", e))?;

    if bytes_read <= 0 {
        return Err("Invalid empty command".to_string());
    }

    let line_args: Vec<&str> = line.trim().split_whitespace().collect();

    let command = *line_args.get(0)
    .ok_or_else(||"Invalid empty command".to_string())?;



       match command {
        //CORE COMMANDS
        "next" | "nx" => {

        },
        "previous" | "pv" => {

        },
        "pause" | "p" => {

        },
        //NAVIGATION COMMANDS

        "current" => {

        },
        "chplaylist" => {
           let _ = check_args(line_args, 1)?;

        },
        "lsplaylist" => {

        },
        "qnext" => {
            let _ = check_args(line_args, 1)?;
        },
        "playsong" => {
            let _ = check_args(line_args, 1)?;
        },
        _ => {
           return Err(format!("{} is not a command", command))
        }
    }

    Ok(format!("{}",command))
}

fn check_args(args_vec: Vec<&str>, index: usize) -> Result<&str, String>{
    
    if args_vec.get(index).is_none() {
        return Err("This command requires an argument".to_string());
    };
    Ok(args_vec[index])
}