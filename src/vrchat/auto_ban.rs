use crate::config::CONFIG;
use crate::events::{AppEvent, EVENT_BUS};
use crate::listen;
use crate::vrchat::util::extract_avatar_file_id;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::io::{self, BufRead};
use tracing::{error, info};
use vrchatapi::apis;
use vrchatapi::apis::configuration::Configuration;
use vrchatapi::models::{BanGroupMemberRequest, User};

async fn process_user(config: &Configuration, user_id: String, user: User) -> Result<()> {
    let group_id = CONFIG
        .group_id
        .clone()
        .context("group_id config variable not set")?;

    let avatar_id = match extract_avatar_file_id(&user)? {
        Some(id) => id,
        _ => return Ok(()),
    };

    let banned_avatars = load_avatar_list().context("Failed to load avatar list")?;

    if !banned_avatars.contains(&avatar_id) {
        return Ok(());
    }

    let ban_request = BanGroupMemberRequest::new(user_id.clone());
    apis::groups_api::ban_group_member(config, group_id.as_str(), ban_request)
        .await
        .context("Failed to ban user")?;

    info!("Banned {} from the group", user_id);

    EVENT_BUS
        .publish(AppEvent::OnAutoBanned(user_id, avatar_id))
        .await;

    Ok(())
}

fn load_avatar_list() -> io::Result<HashSet<String>> {
    let avatar_file = CONFIG
        .avatars_file
        .clone()
        .unwrap_or("avatars.txt".to_string());

    let file = std::fs::File::open(avatar_file.as_str())?;
    let reader = io::BufReader::new(file);
    Ok(reader.lines().filter_map(Result::ok).collect())
}

pub fn auto_ban(auth_config: &apis::configuration::Configuration) {
    let auth_config_clone = auth_config.clone();

    listen!(
        AppEvent::OnPlayerJoined(user_id, user) => {
          if let Err(err) = process_user(&auth_config_clone, user_id.clone(), user).await {
            error!("Failed to process user {}, err: {:#}", user_id, err);
          };
        },
        AppEvent::OnAvatarChanged(user_id, user) => {
          if let Err(err) = process_user(&auth_config_clone, user_id.clone(), user).await {
            error!("Failed to process user {}, err: {:#}", user_id, err);
          };
        }
    );
}
