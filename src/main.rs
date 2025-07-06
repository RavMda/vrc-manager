use anyhow::Result;
use tracing::error;

use crate::config::CONFIG;

mod config;
mod events;
mod log_parser;
mod logging;
mod vrchat;

#[tokio::main]
async fn main() -> Result<()> {
    logging::init();

    tokio::spawn(async move {
        if let Err(err) = log_parser::start_loop().await {
            error!("Log parser failed: {:#}", err);
        }
    });

    let auth_config = vrchat::auth().await?;

    if CONFIG.auto_ban {
        vrchat::auto_ban(&auth_config);
    }

    if CONFIG.auto_invite.enabled {
        vrchat::auto_invite(&auth_config);
    }

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
