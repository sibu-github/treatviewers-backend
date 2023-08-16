use jsonwebtoken::{
    decode, encode, errors::Result as JwtResult, DecodingKey, EncodingKey, Header, Validation,
};
use lazy_static::lazy_static;

use crate::{models::JwtClaims, utils::get_epoch_ts};

lazy_static! {
    pub static ref JWT_KEYS: JwtKeys = JwtKeys::new();
}

pub struct JwtKeys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl JwtKeys {
    fn new() -> Self {
        let secret = std::env::var("JWT_SECRET_KEY").unwrap_or("my_secret".to_string());
        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn generate_token(
        &self,
        id: u32,
        name: Option<String>,
        is_admin: bool,
    ) -> JwtResult<String> {
        let jwt_expiry = std::env::var("JWT_EXPIRY").unwrap_or_default();
        let jwt_expiry = jwt_expiry.parse::<usize>().unwrap_or(3600);
        let jwt_expiry = get_epoch_ts() as usize + jwt_expiry;
        self.sign(id, name, is_admin, jwt_expiry)
    }

    pub fn generate_refresh_token(&self, id: u32, name: Option<String>) -> JwtResult<String> {
        let jwt_expiry = std::env::var("REFRESH_TOKEN_EXPIRY").unwrap_or_default();
        let jwt_expiry = jwt_expiry.parse::<usize>().unwrap_or(24 * 3600);
        let jwt_expiry = get_epoch_ts() as usize + jwt_expiry;
        self.sign(id, name, false, jwt_expiry)
    }

    fn sign(&self, id: u32, name: Option<String>, is_admin: bool, exp: usize) -> JwtResult<String> {
        let claims = JwtClaims::new(id, name, is_admin, exp);
        encode(&Header::default(), &claims, &self.encoding)
    }

    pub fn extract_claims(&self, token: &str) -> Option<JwtClaims> {
        let token_data =
            decode::<JwtClaims>(&token, &self.decoding, &Validation::default()).ok()?;
        Some(token_data.claims)
    }
}
