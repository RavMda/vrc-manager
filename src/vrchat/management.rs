use crate::events::AppEvent;
use crate::listen;
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::io::{self, BufRead};
use url::Url;
use vrchatapi::apis;
use vrchatapi::models::BanGroupMemberRequest;

static AVATAR_FILE: Lazy<String> =
    Lazy::new(|| std::env::var("AVATAR_FILE").unwrap_or("avatars.txt".to_string()));

static GROUP_ID: Lazy<String> =
    Lazy::new(|| std::env::var("GROUP_ID").expect("GROUP_ID environment variable not set"));

async fn process_user(config: &apis::configuration::Configuration, user_id: String) -> Result<()> {
    let user = apis::users_api::get_user(config, &user_id)
        .await
        .context("Failed to retrieve user data")?;

    let avatar_id =
        extract_avatar_id(&user.current_avatar_image_url).context("Failed to extract avatar ID")?;

    let banned_avatars = load_avatar_list().context("Failed to load avatar list")?;

    if !banned_avatars.contains(&avatar_id) {
        return Ok(());
    }

    let ban_request = BanGroupMemberRequest::new(user_id.clone());
    apis::groups_api::ban_group_member(config, GROUP_ID.as_str(), ban_request)
        .await
        .context("Failed to ban user")?;

    Ok(())
}

fn extract_avatar_id(url: &str) -> Option<String> {
    Url::parse(url)
        .ok()?
        .path_segments()?
        .nth(3)
        .map(|s| s.to_string())
}

fn load_avatar_list() -> io::Result<HashSet<String>> {
    let file = std::fs::File::open(AVATAR_FILE.as_str())?;
    let reader = io::BufReader::new(file);
    Ok(reader.lines().filter_map(Result::ok).collect())
}

pub fn avatar_autoban(auth_config: &apis::configuration::Configuration) {
    let auth_config_clone = auth_config.clone();

    listen!(
        AppEvent::OnPlayerJoined(user_id) => {
          if let Err(err) = process_user(&auth_config_clone, user_id.clone()).await {
            eprintln!("Failed to process user {}, err: {}", user_id, err);
          };
        }
    );

    let auth_config_clone2 = auth_config.clone();

    listen!(
        AppEvent::OnAvatarChanged(user_id) => {
          if let Err(err) = process_user(&auth_config_clone2, user_id.clone()).await {
            eprintln!("Failed to process user {}, err: {}", user_id, err);
          };
        }
    );
}
