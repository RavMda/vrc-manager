use anyhow::{Context, Result};
use url::Url;
use vrchatapi::models::User;

pub fn extract_avatar_file_id(user: &User) -> Result<Option<String>> {
    if user.profile_pic_override != "".to_string() {
        return Ok(None);
    }

    let avatar_file_id = Url::parse(&user.current_avatar_image_url)
        .ok()
        .context("Failed to parse avatar URL")?
        .path_segments()
        .context("Failed to get path segments")?
        .nth(3)
        .map(|s| s.to_string())
        .context("Failed to extract avatar ID")?;

    Ok(Some(avatar_file_id))
}
