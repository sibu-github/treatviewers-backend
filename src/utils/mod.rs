use axum::http::Uri;
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::error::Error as MongoError;
use mongodb::error::Result as MongoResult;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument, UpdateModifications};

use crate::{config::AppError, constants::*, models::*};

#[cfg(test)]
pub mod test_helper;

mod import_double;
mod misc;
mod token;
mod unprotected_route;

pub use misc::get_epoch_ts;

#[cfg_attr(test, mockall_double::double)]
use crate::config::database::DbClient;

#[cfg_attr(test, mockall_double::double)]
use crate::config::database_session::DbSession;

pub struct Utility;

#[cfg_attr(test, mockall::automock)]
impl Utility {
    pub fn new() -> Self {
        Self
    }
    pub fn get_epoch_ts(&self) -> u64 {
        misc::get_epoch_ts()
    }
    pub async fn get_seq_nxt_val(&self, seq_id: &str, db: &DbClient) -> anyhow::Result<u32> {
        misc::get_seq_nxt_val(seq_id, db).await
    }
    pub fn decode_token(&self, token: &str) -> Result<JwtClaims, AppError> {
        token::decode_token(token)
    }
    pub fn generate_token(&self, user_id: u32, name: Option<String>) -> anyhow::Result<String> {
        token::generate_token(user_id, name)
    }
    pub fn generate_token_admin(
        &self,
        user_id: u32,
        name: Option<String>,
    ) -> anyhow::Result<String> {
        token::generate_token_admin(user_id, name)
    }
}
