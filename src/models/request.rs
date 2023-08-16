use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// request schema for Add Balanace Init request
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct AddBalInitReq {
    #[validate(range(min = 1))]
    pub amount: u64,
}
