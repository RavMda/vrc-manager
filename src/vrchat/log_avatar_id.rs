use tracing::info;
use vrchatapi::models::User;

use crate::{events::AppEvent, listen, vrchat::util::extract_avatar_file_id};

async fn handle_event(user_id: String, user: User) {
    let avatar_file_id = match extract_avatar_file_id(&user).unwrap_or(None) {
        Some(avatar_file_id) => avatar_file_id,
        _ => return,
    };

    info!(
        "Avatar File ID of {} ({}): {}",
        user.display_name, user_id, avatar_file_id
    );
}

pub fn log_avatar_id() {
    listen!(
        AppEvent::OnPlayerJoined(user_id, user) => {
          handle_event(user_id, user).await;
        },
        AppEvent::OnAvatarChanged(user_id, user) => {
          handle_event(user_id, user).await;
        }
    )
}
