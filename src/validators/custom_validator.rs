use validator::ValidationError;

/// Custom validator function to check phone number
pub fn validate_phonenumber(phone: &str) -> Result<(), ValidationError> {
    // phone must be 10 digits long
    if phone.len() != 10 {
        let mut err = ValidationError::new("phone");
        err.message =
            Some(format!("Phone must be 10 digits. Invalid phone received: {phone}").into());
        return Err(err);
    }
    // phone must be all numeric chars
    if !phone.chars().all(|ch| ch.is_ascii_digit()) {
        let mut err = ValidationError::new("phone");
        err.message =
            Some(format!("Phone must be all digits. Invalid phone received: {phone}").into());
        return Err(err);
    }

    Ok(())
}
