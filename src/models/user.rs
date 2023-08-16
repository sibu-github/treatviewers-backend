use mongodb::bson::Bson;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::Money;

#[derive(Debug, Deserialize)]
pub struct JwkKeys {
    pub kid: String,
    pub n: String,
    pub e: String,
}

#[derive(Debug, Deserialize)]
pub struct JwksResp {
    pub keys: Vec<JwkKeys>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginTokenData {
    pub name: String,
    pub email: String,
    pub profile_pic: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LoginScheme {
    #[default]
    OtpBased,
    Google,
    Facebook,
}

impl LoginScheme {
    pub fn to_bson(&self) -> anyhow::Result<Bson> {
        let bson = mongodb::bson::to_bson(self)?;
        Ok(bson)
    }
}

#[derive(Debug, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: u32,
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_pic: Option<String>,

    pub login_scheme: LoginScheme,
    pub is_active: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login_time: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_used_referral_code: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_referral_code: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub referral_code: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub referred_by: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_played: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub contest_won: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_earning: Option<Money>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_ts: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_ts: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fcm_tokens: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminUser {
    pub id: u32,
    pub name: String,
    pub phone: String,
    pub is_active: bool,
}
