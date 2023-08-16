use crate::{constants::GOOGLE_JWKS_URI, models::JwksResp};

pub async fn google_jwks() -> anyhow::Result<JwksResp> {
    let jwks_resp = reqwest::get(GOOGLE_JWKS_URI)
        .await?
        .json::<JwksResp>()
        .await?;
    Ok(jwks_resp)
}
