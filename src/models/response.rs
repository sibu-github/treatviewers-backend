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

/// response schema for Add Balance Init
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddBalInitRes {
    pub success: bool,
    pub transaction_id: String,
    pub app_upi_id: String,
}
