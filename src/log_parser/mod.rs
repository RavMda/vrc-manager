use anyhow::{Context, Result};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::Mutex;

use crate::config::CONFIG;
use crate::events::{AppEvent, BUS};

static JOIN_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?x)
        ^(\d{4}\.\d{2}\.\d{2}\ \d{2}:\d{2}:\d{2})  # Timestamp
        \s+\w+\s+-\s+                             # Log level and hyphen
        \[Behaviour\]\sOnPlayerJoined\s           # Event
        ([^\(]+)\s\(                              # Username
        (usr_[0-9a-fA-F-]+)                      # User ID
        ",
    )
    .unwrap()
});

static LEAVE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?x)
        ^(\d{4}\.\d{2}\.\d{2}\ \d{2}:\d{2}:\d{2})  # Timestamp
        \s+\w+\s+-\s+                             # Log level and hyphen
        \[Behaviour\]\sOnPlayerLeft\s             # Event
        ([^\(]+)\s\(                              # Username
        (usr_[0-9a-fA-F-]+)                      # User ID
        ",
    )
    .unwrap()
});

static AVATAR_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?x)
        ^(\d{4}\.\d{2}\.\d{2}\ \d{2}:\d{2}:\d{2})  # Timestamp
        \s+\w+\s+-\s+                             # Log level and hyphen
        \[Behaviour\]\sSwitching\s                # Event
        ([^\s]+)\s+to\savatar\s                   # Username
        ",
    )
    .unwrap()
});

pub async fn start_loop() -> Result<()> {
    let log_dir = get_vrchat_log_dir()?;
    let mut current_log_path = find_latest_log().await?;
    let program_start = Local::now();

    let mut file = fs::File::open(&current_log_path).await?;
    let mut last_position = file.seek(std::io::SeekFrom::End(0)).await?;
    let mut buffer = String::new();
    let user_map = Mutex::new(HashMap::new());

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        },
        notify::Config::default(),
    )?;

    watcher.watch(&log_dir, RecursiveMode::NonRecursive)?;
    watcher.watch(&current_log_path, RecursiveMode::NonRecursive)?;

    loop {
        let event = rx.recv().await;

        match event {
            Some(event) => {
                if event.paths.iter().any(|p| p == &log_dir) {
                    if let EventKind::Create(_) = event.kind {
                        if let Ok(newest_log) = find_latest_log().await {
                            if newest_log != current_log_path {
                                current_log_path = newest_log.clone();
                                file = fs::File::open(&newest_log).await?;
                                last_position = file.seek(std::io::SeekFrom::End(0)).await?;
                                buffer.clear();
                                watcher.unwatch(&current_log_path)?;
                                watcher.watch(&newest_log, RecursiveMode::NonRecursive)?;
                            }
                        }
                    }
                } else if event.paths.iter().any(|p| p == &current_log_path) {
                    if let EventKind::Modify(_) = event.kind {
                        let metadata = fs::metadata(&current_log_path).await?;
                        let current_len = metadata.len();

                        if current_len > last_position {
                            file = reopen_log_file(&current_log_path).await?;
                            file.seek(std::io::SeekFrom::Start(last_position)).await?;

                            let mut chunk = Vec::new();
                            file.take(current_len - last_position)
                                .read_to_end(&mut chunk)
                                .await?;

                            process_log_chunk(
                                String::from_utf8_lossy(&chunk).into_owned(),
                                &mut buffer,
                                program_start,
                                &user_map,
                            )
                            .await?;

                            last_position = current_len;
                        }
                    }
                }
            }
            None => break,
        }
    }

    Ok(())
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
    user_map: &Mutex<HashMap<String, String>>,
) -> Result<()> {
    let data = std::mem::take(buffer) + &chunk;

    for line in data.lines() {
        if let Some(captures) = JOIN_PATTERN.captures(line) {
            let timestamp_str = captures.get(1).unwrap().as_str();
            let username = captures.get(2).unwrap().as_str().trim();
            let user_id = captures.get(3).unwrap().as_str();

            if let Ok(timestamp) = NaiveDateTime::parse_from_str(timestamp_str, "%Y.%m.%d %H:%M:%S")
            {
                let datetime = Local.from_local_datetime(&timestamp).unwrap();
                if datetime > program_start {
                    {
                        let mut map = user_map.lock().await;
                        map.insert(username.to_string(), user_id.to_string());
                    }

                    BUS.publish(AppEvent::OnPlayerJoined(user_id.into())).await;
                }
            }
        } else if let Some(captures) = LEAVE_PATTERN.captures(line) {
            let timestamp_str = captures.get(1).unwrap().as_str();
            let username = captures.get(2).unwrap().as_str().trim();
            let user_id = captures.get(3).unwrap().as_str();

            if let Ok(timestamp) = NaiveDateTime::parse_from_str(timestamp_str, "%Y.%m.%d %H:%M:%S")
            {
                let datetime = Local.from_local_datetime(&timestamp).unwrap();
                if datetime > program_start {
                    let mut map = user_map.lock().await;
                    map.remove(username);

                    BUS.publish(AppEvent::OnPlayerLeft(user_id.into())).await;
                }
            }
        } else if let Some(captures) = AVATAR_PATTERN.captures(line) {
            let timestamp_str = captures.get(1).unwrap().as_str();
            let username = captures.get(2).unwrap().as_str().trim();

            if let Ok(timestamp) = NaiveDateTime::parse_from_str(timestamp_str, "%Y.%m.%d %H:%M:%S")
            {
                let datetime = Local.from_local_datetime(&timestamp).unwrap();
                if datetime > program_start {
                    let map = user_map.lock().await;
                    if let Some(user_id) = map.get(username) {
                        BUS.publish(AppEvent::OnAvatarChanged(user_id.into())).await;
                    }
                }
            }
        }
    }

    if let Some(last_newline) = data.rfind('\n') {
        buffer.push_str(&data[last_newline + 1..]);
    } else {
        *buffer = data;
    }

    Ok(())
}

fn get_vrchat_log_dir() -> Result<PathBuf> {
    if let Some(custom_dir) = CONFIG.custom_log_dir.clone() {
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
