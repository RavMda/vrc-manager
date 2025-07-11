use anyhow::{Result, anyhow};
use tracing::error;
use vrchatapi::{
    apis::{self, configuration::Configuration},
    models::{EitherUserOrTwoFactor, User},
};

use crate::{
    events::{AppEvent, EVENT_BUS},
    listen,
};

async fn fetch_user_data(config: &Configuration, user_id: &str) -> Result<Option<User>> {
    let current_user = match apis::authentication_api::get_current_user(config).await {
        Ok(EitherUserOrTwoFactor::CurrentUser(user)) => user,
        _ => return Err(anyhow!("Failed to get current user")),
    };

    let user = apis::users_api::get_user(config, user_id)
        .await
        .map_err(|e| anyhow!(e))?;

    if current_user.id == user.id {
        Ok(None)
    } else {
        Ok(Some(user))
    }
}

async fn handle_event(
    auth_config: Configuration,
    user_id: String,
    event_constructor: fn(String, User) -> AppEvent,
) {
    match fetch_user_data(&auth_config, &user_id).await {
        Ok(Some(user)) => EVENT_BUS.publish(event_constructor(user_id, user)).await,
        Ok(None) => (),
        Err(e) => error!("Failed to process user {}: {:#}", user_id, e),
    }
}

pub fn init(auth_config: &Configuration) {
    let auth_config = auth_config.clone();

    listen!(
        AppEvent::OnPlayerJoinedRaw(user_id) => {
            handle_event(auth_config.clone(), user_id, AppEvent::OnPlayerJoined).await;
        },
        AppEvent::OnAvatarChangedRaw(user_id) => {
            handle_event(auth_config.clone(), user_id, AppEvent::OnAvatarChanged).await;
        },
        AppEvent::OnPlayerLeftRaw(user_id) => {
            handle_event(auth_config.clone(), user_id, AppEvent::OnPlayerLeft).await;
        }
    );
}
