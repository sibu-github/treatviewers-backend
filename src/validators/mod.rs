mod custom_validator;
mod validate_extra;

pub use custom_validator::*;
pub use validate_extra::ValidateExtra;

use crate::config::AppError;

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
}
