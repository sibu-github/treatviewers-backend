use std::sync::Arc;

use axum::async_trait;

use crate::config::{AppError, AppState};

#[async_trait]
pub trait ValidateExtra {
    async fn validate_extra(
        &self,
        _s: Arc<AppState>,
        user_id: Option<u32>,
    ) -> Result<(), AppError> {
        Ok(())
    }
}
