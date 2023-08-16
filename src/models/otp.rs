use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Otp {
    pub user_id: u32,
    pub otp: String,
    pub valid_till: u64,
    pub is_used: bool,
    pub update_ts: u64,
}
