use serde_json::Value as JsonValue;

use crate::models::*;

mod fb_me;
mod google_jwks;
mod sms;

#[cfg(test)]
use mockall::automock;

pub struct ExternalApi;

#[cfg_attr(test, automock)]
impl ExternalApi {
    pub fn new() -> Self {
        Self
    }
    pub async fn send_sms(&self, phone: &str, message: &str) -> anyhow::Result<()> {
        sms::send_sms(phone, message).await
    }
    pub async fn google_jwks(&self) -> anyhow::Result<JwksResp> {
        google_jwks::google_jwks().await
    }
    pub async fn fb_me(&self, fb_token: &str) -> anyhow::Result<JsonValue> {
        fb_me::fb_me(fb_token).await
    }
}
