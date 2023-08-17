use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::impl_validate_extra;

/// request schema for Add Balanace Init request
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct AddBalInitReq {
    #[validate(range(min = 1))]
    pub amount: u64,
}
impl_validate_extra!(AddBalInitReq);

/// request schema for Add Balance Init request
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddBalEndReq {
    #[validate(range(min = 1))]
    pub amount: u64,
    pub transaction_id: ObjectId,
    pub is_successful: bool,
    pub error_reason: Option<String>,
    pub tracking_id: Option<String>,
}
