use crate::config::CONFIG;
use crate::events::AppEvent;
use crate::listen;
use anyhow::{Context, Result};
use vrchatapi::apis;
use vrchatapi::models::CreateGroupInviteRequest;

async fn process_user(config: &apis::configuration::Configuration, user_id: String) -> Result<()> {
    let group_id = CONFIG
        .group_id
        .clone()
        .context("group_id config variable is not set")?;

    let invite_request = CreateGroupInviteRequest::new(user_id.clone());
    apis::groups_api::create_group_invite(config, group_id.as_str(), invite_request)
        .await
        .context("Failed to invite user")?;

    println!("Invited {} to the group", user_id);

    Ok(())
}

pub fn auto_invite(auth_config: &apis::configuration::Configuration) {
    let auth_config_clone = auth_config.clone();

    listen!(
        AppEvent::OnPlayerJoined(user_id) => {
          if let Err(err) = process_user(&auth_config_clone, user_id.clone()).await {
            eprintln!("Failed to process user {}, err: {:#}", user_id, err);
          };
        }
    );
}
