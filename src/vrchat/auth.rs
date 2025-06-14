use anyhow::{Context, Result, bail};
use reqwest::cookie::CookieStore;
use std::io::{self, Write};
use std::sync::Arc;
use url::Url;
pub use vrchatapi::apis;
use vrchatapi::apis::configuration::Configuration;
use vrchatapi::models::{
    CurrentUser, EitherUserOrTwoFactor, TwoFactorAuthCode, TwoFactorEmailCode,
};

pub async fn auth() -> Result<Configuration> {
    let username = read_user_input("Enter your username: ");
    let cookie_filename = format!("{}.cookie", username);
    let cookie_jar = Arc::new(reqwest::cookie::Jar::default());

    let mut config = Configuration::default();
    config.user_agent = Some("VRCAutoBan/0.0.1 49620461+RavMda@users.noreply.github.com".into());
    config.client = reqwest::Client::builder()
        .cookie_provider(cookie_jar.clone())
        .build()
        .context("Failed to build HTTP client")?;

    if try_existing_cookie(&username, &config, &cookie_jar).await? {
        return Ok(config);
    }

    perform_password_auth(&username, &mut config, &cookie_jar, &cookie_filename).await
}

async fn try_existing_cookie(
    username: &str,
    config: &Configuration,
    cookie_jar: &Arc<reqwest::cookie::Jar>,
) -> Result<bool> {
    if load_cookies(username, cookie_jar).is_err() {
        return Ok(false);
    }

    match apis::authentication_api::get_current_user(config).await {
        Ok(EitherUserOrTwoFactor::CurrentUser(user)) => {
            println!(
                "Logged in with existing cookie as {}",
                user.username.as_deref().unwrap_or("Unknown")
            );
            Ok(true)
        }
        _ => {
            std::fs::remove_file(format!("{}.cookie", username)).ok();
            Ok(false)
        }
    }
}

async fn perform_password_auth(
    username: &str,
    config: &mut Configuration,
    cookie_jar: &Arc<reqwest::cookie::Jar>,
    cookie_filename: &str,
) -> Result<Configuration> {
    let password = read_user_input("Enter your password: ");
    config.basic_auth = Some((username.into(), Some(password)));

    loop {
        match apis::authentication_api::get_current_user(config).await {
            Ok(EitherUserOrTwoFactor::CurrentUser(user)) => {
                save_cookies(cookie_jar, cookie_filename)?;
                println!("Logged in as {}", user.display_name);
                break;
            }
            Ok(EitherUserOrTwoFactor::RequiresTwoFactorAuth(req)) => {
                handle_two_factor_auth(config, req).await?;
            }
            Err(e) => {
                eprintln!("Authentication failed: {}", e);
                std::process::exit(1);
            }
        }
    }

    Ok(config.clone())
}

async fn handle_two_factor_auth(
    config: &Configuration,
    requirements: vrchatapi::models::current_user::RequiresTwoFactorAuth,
) -> Result<()> {
    if requirements
        .requires_two_factor_auth
        .contains(&"emailOtp".to_string())
    {
        let code = read_user_input("Enter Email 2FA code: ");
        apis::authentication_api::verify2_fa_email_code(config, TwoFactorEmailCode::new(code))
            .await
            .context("Email 2FA verification failed")?;
    } else {
        let code = read_user_input("Enter Authenticator 2FA code: ");
        apis::authentication_api::verify2_fa(config, TwoFactorAuthCode::new(code))
            .await
            .context("Authenticator 2FA verification failed")?;
    }
    Ok(())
}

pub async fn get_current_user(config: &Configuration) -> Result<CurrentUser> {
    match apis::authentication_api::get_current_user(config).await? {
        EitherUserOrTwoFactor::CurrentUser(user) => Ok(user),
        EitherUserOrTwoFactor::RequiresTwoFactorAuth(_) => {
            bail!("Two-factor authentication required")
        }
    }
}

fn read_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().expect("Failed to flush stdout");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().to_string()
}

fn load_cookies(username: &str, jar: &reqwest::cookie::Jar) -> Result<()> {
    let url = Url::parse("https://api.vrchat.cloud").context("Failed  to parse URL")?;
    let content = std::fs::read_to_string(format!("{}.cookie", username))?;

    for cookie_str in content.split(';').filter(|s| !s.trim().is_empty()) {
        let cookie = format!("{}; Domain=api.vrchat.cloud; Path=/", cookie_str.trim());
        jar.add_cookie_str(&cookie, &url);
    }

    Ok(())
}

fn save_cookies(cookie_jar: &reqwest::cookie::Jar, cookie_filename: &str) -> Result<()> {
    let url = Url::parse("https://api.vrchat.cloud").context("Failed  to parse URL")?;
    let cookies = cookie_jar.cookies(&url).context("Failed to get cookies")?;
    let cookie_str = cookies
        .to_str()
        .context("Failed to convert cookies to string")?;
    std::fs::write(cookie_filename, cookie_str).context("Failed to save cookies")?;
    Ok(())
}
