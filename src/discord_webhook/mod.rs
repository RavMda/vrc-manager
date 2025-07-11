use crate::{config::CONFIG, events::AppEvent, listen};
use reqwest::Client;
use serde::Serialize;
use tracing::error;
use vrchatapi::apis::configuration::Configuration as ApiConfig;
use vrchatapi::apis::users_api;

pub fn init(auth_config: &ApiConfig) {
    if CONFIG.discord_webhook.log_on_auto_ban {
        let auth_config = auth_config.clone();
        listen!(
            AppEvent::OnAutoBanned(user_id, avatar_id) => {
                handle_auto_ban(&auth_config, user_id, avatar_id).await;
            }
        );
    }

    if CONFIG.discord_webhook.log_on_auto_invite {
        let auth_config = auth_config.clone();
        listen!(
            AppEvent::OnAutoInvited(user_id) => {
                handle_auto_invite(&auth_config, user_id).await;
            }
        );
    }

    if CONFIG.discord_webhook.log_on_player_joined {
        let auth_config = auth_config.clone();
        listen!(
            AppEvent::OnPlayerJoined(user_id) => {
                handle_player_joined(&auth_config, user_id).await;
            }
        );
    }

    if CONFIG.discord_webhook.log_on_player_left {
        let auth_config = auth_config.clone();
        listen!(
            AppEvent::OnPlayerLeft(user_id) => {
                handle_player_left(&auth_config, user_id).await;
            }
        );
    }
}

#[derive(Serialize)]
struct Embed {
    title: String,
    description: String,
    fields: Vec<Field>,
    thumbnail: Thumbnail,
    color: u32,
}

#[derive(Serialize)]
struct Field {
    name: String,
    value: String,
    inline: bool,
}

#[derive(Serialize)]
struct Thumbnail {
    url: String,
}

#[derive(Serialize)]
struct WebhookPayload {
    username: String,
    avatar_url: String,
    embeds: Vec<Embed>,
}

async fn handle_auto_ban(auth_config: &ApiConfig, user_id: String, avatar_id: String) {
    send_embed(auth_config, user_id, move |user| Embed {
        title: "User Banned".into(),
        description: format!(
            "User **{}** has been banned for using avatar ID: `{}`",
            user.display_name, avatar_id
        ),
        fields: vec![Field {
            name: "User ID".into(),
            value: user.id,
            inline: false,
        }],
        thumbnail: Thumbnail {
            url: user.current_avatar_thumbnail_image_url,
        },
        color: 0xFF0000,
    })
    .await;
}

async fn handle_auto_invite(auth_config: &ApiConfig, user_id: String) {
    send_embed(auth_config, user_id, |user| Embed {
        title: "User Invited".into(),
        description: format!(
            "User **{}** has been automatically invited",
            user.display_name
        ),
        fields: vec![Field {
            name: "User ID".into(),
            value: user.id,
            inline: false,
        }],
        thumbnail: Thumbnail {
            url: user.current_avatar_thumbnail_image_url,
        },
        color: 0x0000FF,
    })
    .await;
}

async fn handle_player_joined(auth_config: &ApiConfig, user_id: String) {
    send_embed(auth_config, user_id, |user| Embed {
        title: "Player Joined".into(),
        description: format!("**{}** has joined the instance!", user.display_name),
        fields: vec![Field {
            name: "User ID".into(),
            value: user.id,
            inline: false,
        }],
        thumbnail: Thumbnail {
            url: user.current_avatar_thumbnail_image_url,
        },
        color: 0x00FF00,
    })
    .await;
}

async fn handle_player_left(auth_config: &ApiConfig, user_id: String) {
    send_embed(auth_config, user_id, |user| Embed {
        title: "Player Left".into(),
        description: format!("**{}** has left the instance", user.display_name),
        fields: vec![Field {
            name: "User ID".into(),
            value: user.id,
            inline: false,
        }],
        thumbnail: Thumbnail {
            url: user.current_avatar_thumbnail_image_url,
        },
        color: 0xFFA500,
    })
    .await;
}

async fn send_embed<F>(auth_config: &ApiConfig, user_id: String, embed_builder: F)
where
    F: FnOnce(vrchatapi::models::User) -> Embed,
{
    let user = match users_api::get_user(auth_config, &user_id).await {
        Ok(u) => u,
        Err(e) => {
            error!("Failed to fetch user {}: {}", user_id, e);
            return;
        }
    };

    let payload = WebhookPayload {
        username: CONFIG.discord_webhook.username.clone(),
        avatar_url: CONFIG.discord_webhook.avatar_url.clone(),
        embeds: vec![embed_builder(user)],
    };

    let client = Client::new();
    if let Err(e) = client
        .post(&CONFIG.discord_webhook.url)
        .json(&payload)
        .send()
        .await
    {
        error!("Webhook execution failed: {}", e);
    }
}
