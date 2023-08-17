mod add_bal;
mod custom_validator;
mod validate_extra;

pub use custom_validator::*;
pub use validate_extra::ValidateExtra;

use crate::{config::AppError, helpers::Helpers, import_double, models::*};

import_double!(DbClient);

pub struct Validators;

#[cfg_attr(test, mockall::automock)]
impl Validators {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_phonenumber(&self, phone: &str) -> Result<(), AppError> {
        custom_validator::validate_phonenumber(phone)
            .map_err(|e| AppError::BadRequest(e.to_string()))
    }

    pub async fn validate_add_bal_transaction(
        &self,
        db: &DbClient,
        helper: &Helpers,
        user_id: u32,
        body: &AddBalEndReq,
    ) -> Result<(), AppError> {
        add_bal::validate_add_bal_transaction(db, helper, user_id, body).await
    }
}
