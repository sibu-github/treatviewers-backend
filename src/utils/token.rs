use jsonwebtoken::{decode, Validation};

use crate::{
    config::{jwt::JWT_KEYS, AppError},
    models::JwtClaims,
};

pub fn decode_token(token: &str) -> Result<JwtClaims, AppError> {
    let token_data = decode::<JwtClaims>(token, &JWT_KEYS.decoding, &Validation::default())
        .map_err(|e| {
            tracing::debug!("{:?}", e);
            AppError::Auth(e.to_string())
        })?;
    Ok(token_data.claims)
}

pub fn generate_token(user_id: u32, name: Option<String>) -> anyhow::Result<String> {
    let token = JWT_KEYS.generate_token(user_id, name, false)?;
    Ok(token)
}

pub fn generate_token_admin(user_id: u32, name: Option<String>) -> anyhow::Result<String> {
    let token = JWT_KEYS.generate_token(user_id, name, true)?;
    Ok(token)
}
