use axum::http::Uri;

mod misc;
mod token;
mod unprotected_route;

pub use misc::get_epoch_ts;

#[cfg(test)]
use mockall::automock;

#[cfg(test)]
use mockall_double::double;

#[cfg_attr(test, double)]
use crate::config::database::DbClient;
use crate::{config::AppError, models::*};

pub struct Utility;

#[cfg_attr(test, automock)]
impl Utility {
    pub fn new() -> Self {
        Self
    }
    pub fn is_unprotected_path(&self, uri: &Uri) -> bool {
        unprotected_route::is_unprotected_path(uri)
    }
    pub fn is_admin_only_path(&self, uri: &Uri) -> bool {
        unprotected_route::is_admin_only_path(uri)
    }
    pub async fn get_seq_nxt_val(&self, seq_id: &str, db: &DbClient) -> anyhow::Result<u32> {
        misc::get_seq_nxt_val(seq_id, db).await
    }
    pub fn decode_token(&self, token: &str) -> Result<JwtClaims, AppError> {
        token::decode_token(token)
    }
    pub fn generate_token(user_id: u32, name: Option<String>) -> anyhow::Result<String> {
        token::generate_token(user_id, name)
    }
    pub fn generate_token_admin(user_id: u32, name: Option<String>) -> anyhow::Result<String> {
        token::generate_token_admin(user_id, name)
    }
}
