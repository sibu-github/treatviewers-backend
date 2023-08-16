use serde_json::Value as JsonValue;

use crate::constants::FB_ME_URL;

pub async fn fb_me(fb_token: &str) -> anyhow::Result<JsonValue> {
    let url = format!(
        "{}?access_token={}&fields=id,name,email,picture",
        FB_ME_URL, fb_token
    );
    let res = reqwest::get(&url).await?.json::<JsonValue>().await?;
    Ok(res)
}
