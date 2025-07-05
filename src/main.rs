use anyhow::Result;

use crate::config::CONFIG;

pub mod config;
pub mod events;
pub mod log_parser;
pub mod vrchat;

#[tokio::main]
async fn main() -> Result<()> {
    tokio::spawn(async move {
        if let Err(err) = log_parser::start_loop().await {
            eprintln!("Log parser failed: {:#}", err);
        }
    });

    let auth_config = vrchat::auth().await?;

    if CONFIG.auto_ban {
        vrchat::auto_ban(&auth_config);
    }

    if CONFIG.auto_invite {
        vrchat::auto_invite(&auth_config);
    }

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
