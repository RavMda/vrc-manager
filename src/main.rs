use anyhow::Result;
use tokio::sync::mpsc;

pub mod log_parser;
pub mod vrchat;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let (tx, mut rx) = mpsc::channel(100);

    tokio::spawn(async move {
        if let Err(err) = log_parser::start_loop(tx).await {
            eprintln!("Log parser failed: {}", err);
        }
    });

    let config = vrchat::auth().await?;

    while let Some(res) = rx.recv().await {
        //println!("Received user ID: {}", res.user_id);

        if let Err(err) = vrchat::management::process_user(&config, res.user_id.clone()).await {
            eprintln!("Failed to process user {}, err: {}", res.user_id, err);
        };
    }

    Ok(())
}
