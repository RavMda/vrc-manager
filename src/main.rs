use anyhow::Result;

pub mod events;
pub mod log_parser;
pub mod vrchat;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tokio::spawn(async move {
        if let Err(err) = log_parser::start_loop().await {
            eprintln!("Log parser failed: {}", err);
        }
    });

    let auth_config = vrchat::auth().await?;

    vrchat::management::avatar_autoban(&auth_config);

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
