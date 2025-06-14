use anyhow::{Context, Result};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::mpsc::Sender;
use tokio::time;

static LOG_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?x)
        ^(\d{4}\.\d{2}\.\d{2}\ \d{2}:\d{2}:\d{2})  # Timestamp
        .*OnPlayerJoined.*                          # Event
        (usr_[0-9a-fA-F-]+)                        # User ID
        ",
    )
    .unwrap()
});

static CUSTOM_LOG_DIR: Lazy<Option<String>> = Lazy::new(|| std::env::var("CUSTOM_LOG_DIR").ok());

pub async fn start_loop(tx: Sender<UserIdResult>) -> Result<()> {
    let log_path = find_latest_log().await?;
    let program_start = Local::now();

    let mut file = fs::File::open(&log_path).await?;
    let mut last_position = file.seek(std::io::SeekFrom::End(0)).await?;
    let mut buffer = String::new();

    loop {
        time::sleep(Duration::from_secs(1)).await;

        let metadata = fs::metadata(&log_path).await?;
        let current_len = metadata.len();

        if current_len > last_position {
            file = reopen_log_file(&log_path).await?;
            file.seek(std::io::SeekFrom::Start(last_position)).await?;

            let mut chunk = Vec::new();
            file.take(current_len - last_position)
                .read_to_end(&mut chunk)
                .await?;

            process_log_chunk(
                String::from_utf8_lossy(&chunk).into_owned(),
                &mut buffer,
                program_start,
                &tx,
            )
            .await?;

            last_position = current_len;
        }
    }
}

async fn reopen_log_file(path: &PathBuf) -> Result<fs::File> {
    fs::File::open(path)
        .await
        .with_context(|| format!("Failed to reopen log file: {:?}", path))
}

async fn process_log_chunk(
    chunk: String,
    buffer: &mut String,
    program_start: DateTime<Local>,
    tx: &Sender<UserIdResult>,
) -> Result<()> {
    let data = std::mem::take(buffer) + &chunk;
    let mut user_ids = Vec::new();

    for line in data.lines() {
        if let Some(captures) = LOG_PATTERN.captures(line) {
            let timestamp_str = captures.get(1).unwrap().as_str();
            let user_id = captures.get(2).unwrap().as_str();

            if let Ok(timestamp) = NaiveDateTime::parse_from_str(timestamp_str, "%Y.%m.%d %H:%M:%S")
            {
                let datetime = Local.from_local_datetime(&timestamp).unwrap();
                if datetime > program_start {
                    user_ids.push(user_id.to_string());
                }
            }
        }
    }

    if let Some(last_newline) = data.rfind('\n') {
        buffer.push_str(&data[last_newline + 1..]);
    } else {
        *buffer = data;
    }

    for user_id in user_ids {
        tx.send(UserIdResult { user_id }).await?;
    }

    Ok(())
}

fn get_vrchat_log_dir() -> Result<PathBuf> {
    if let Some(custom_dir) = CUSTOM_LOG_DIR.as_deref() {
        return Ok(PathBuf::from(custom_dir));
    }

    let user_profile = std::env::var("USERPROFILE")
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "USERPROFILE not found"))?;

    Ok(PathBuf::from(user_profile)
        .join("AppData")
        .join("LocalLow")
        .join("VRChat")
        .join("VRChat"))
}

pub async fn find_latest_log() -> Result<PathBuf> {
    let log_dir = get_vrchat_log_dir()?;
    let mut latest_time = None;
    let mut latest_path = None;

    let mut entries = fs::read_dir(log_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                if filename.starts_with("output_log_") && filename.ends_with(".txt") {
                    if let Some(datetime) = parse_log_timestamp(filename) {
                        if latest_time.map_or(true, |t: DateTime<Local>| datetime > t) {
                            latest_time = Some(datetime);
                            latest_path = Some(path);
                        }
                    }
                }
            }
        }
    }

    latest_path.ok_or_else(|| anyhow::anyhow!("No valid log files found"))
}

fn parse_log_timestamp(filename: &str) -> Option<DateTime<Local>> {
    let parts: Vec<&str> = filename.split('_').collect();
    if parts.len() < 4 {
        return None;
    }

    let date_part = parts[2].replace('-', ".");
    let time_part = parts[3].split('.').next()?.replace('-', ":");
    let datetime_str = format!("{} {}", date_part, time_part);

    NaiveDateTime::parse_from_str(&datetime_str, "%Y.%m.%d %H:%M:%S")
        .ok()
        .and_then(|ndt| Local.from_local_datetime(&ndt).single())
}

#[derive(Debug)]
pub struct UserIdResult {
    pub user_id: String,
}
