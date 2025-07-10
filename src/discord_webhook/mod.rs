use crate::{config::CONFIG, events::AppEvent, listen};
use serenity::all::{CreateEmbed, ExecuteWebhook, Http, Webhook};
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

async fn handle_auto_ban(auth_config: &ApiConfig, user_id: String, avatar_id: String) {
    send_embed(auth_config, user_id, |user| {
        CreateEmbed::new()
            .title("User Banned")
            .description(format!(
                "User **{}** has been banned for using avatar ID: `{}`",
                user.display_name.clone(),
                avatar_id
            ))
            .field("User ID", user.id.clone(), false)
            .thumbnail(user.current_avatar_thumbnail_image_url)
            .color(0xFF0000)
    })
    .await;
}

async fn handle_auto_invite(auth_config: &ApiConfig, user_id: String) {
    send_embed(auth_config, user_id, |user| {
        CreateEmbed::new()
            .title("User Invited")
            .description(format!(
                "User **{}** has been automatically invited",
                user.display_name.clone()
            ))
            .field("User ID", user.id.clone(), false)
            .thumbnail(user.current_avatar_thumbnail_image_url)
            .color(0x0000FF)
    })
    .await;
}

async fn handle_player_joined(auth_config: &ApiConfig, user_id: String) {
    send_embed(auth_config, user_id, |user| {
        CreateEmbed::new()
            .title("Player Joined")
            .description(format!(
                "**{}** has joined the instance!",
                user.display_name.clone()
            ))
            .field("User ID", user.id.clone(), false)
            .thumbnail(user.current_avatar_thumbnail_image_url)
            .color(0x00FF00)
    })
    .await;
}

async fn handle_player_left(auth_config: &ApiConfig, user_id: String) {
    send_embed(auth_config, user_id, |user| {
        CreateEmbed::new()
            .title("Player Left")
            .description(format!(
                "**{}** has left the instance",
                user.display_name.clone()
            ))
            .field("User ID", user.id.clone(), false)
            .thumbnail(user.current_avatar_thumbnail_image_url)
            .color(0xFFA500)
    })
    .await;
}

async fn send_embed<F>(auth_config: &ApiConfig, user_id: String, embed_builder: F)
where
    F: FnOnce(vrchatapi::models::User) -> CreateEmbed,
{
    let http = Http::new("");
    let webhook = match Webhook::from_url(&http, &CONFIG.discord_webhook.url).await {
        Ok(w) => w,
        Err(e) => {
            error!("Webhook error: {}", e);
            return;
        }
    };

    let user = match users_api::get_user(auth_config, &user_id).await {
        Ok(u) => u,
        Err(e) => {
            error!("Failed to fetch user {}: {}", user_id, e);
            return;
        }
    };

    let embed = embed_builder(user);

    let builder = ExecuteWebhook::new()
        .username(CONFIG.discord_webhook.username.clone())
        .avatar_url(CONFIG.discord_webhook.avatar_url.clone())
        .embed(embed);

    if let Err(e) = webhook.execute(&http, false, builder).await {
        error!("Webhook execution failed: {}", e);
    }
}
