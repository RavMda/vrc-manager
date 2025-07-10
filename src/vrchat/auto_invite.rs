use crate::config::CONFIG;
use crate::events::{AppEvent, BUS};
use crate::listen;
use anyhow::{Context, Result};
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{error, info};
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

    info!("Invited {} to the group", user_id);

    BUS.publish(AppEvent::OnAutoInvited(user_id)).await;

    Ok(())
}

pub fn auto_invite(auth_config: &apis::configuration::Configuration) {
    let auth_config_clone = auth_config.clone();
    let handles = Arc::new(Mutex::new(HashMap::<String, JoinHandle<()>>::new()));

    let handles_clone = handles.clone();

    listen!(
        AppEvent::OnPlayerJoined(user_id) => {
          let auth_config_clone = auth_config_clone.clone();

          let mut handles_guard = handles_clone.lock().await;

          if let Some(handle) = handles_guard.remove(&user_id) {
              handle.abort();
          }

          let duration = rand::rng().random_range(CONFIG.auto_invite.delay_min..CONFIG.auto_invite.delay_max);
          let sleep_duration = Duration::from_secs(duration as u64);

          let user_id_clone = user_id.clone();
          let task_handles = handles_clone.clone();
          let handle = tokio::spawn(async move {
              tokio::time::sleep(sleep_duration).await;

              if let Err(err) = process_user(&auth_config_clone, user_id_clone.clone()).await {
                  error!("Failed to process user {}: {:#}", user_id_clone, err);
              }

              task_handles.lock().await.remove(&user_id_clone);
          });

          handles_guard.insert(user_id, handle);
        }
    );

    let handles_clone = handles.clone();

    listen!(
        AppEvent::OnPlayerLeft(user_id) => {
          let mut handles_guard = handles_clone.lock().await;
          if let Some(handle) = handles_guard.remove(&user_id) {
              handle.abort();
              info!("Aborted invite timer for {}", user_id);
          }
        }
    );
}
