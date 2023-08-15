use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Response schema for generic response
/// can be used for both success and error response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenericResponse {
    pub success: bool,
    pub message: String,
}

impl GenericResponse {
    pub fn new(success: bool, message: &str) -> Self {
        Self {
            success,
            message: message.to_owned(),
        }
    }

    pub fn json_response(success: bool, message: &str) -> Json<Self> {
        Json(Self::new(success, message))
    }
}
