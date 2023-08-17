use std::sync::Arc;

use axum::async_trait;

use crate::{
    config::{AppError, AppState},
    models::AddBalEndReq,
    validators::ValidateExtra,
};

#[async_trait]
impl ValidateExtra for AddBalEndReq {
    async fn validate_extra(
        &self,
        state: Arc<AppState>,
        user_id: Option<u32>,
    ) -> Result<(), AppError> {
        if !self.is_successful && self.error_reason.is_none() {
            let err = "errorReason is required for failed transaction";
            return Err(AppError::BadRequest(err.into()));
        }
        let user_id = user_id.unwrap_or_default();
        state
            .validators()
            .validate_add_bal_transaction(state.db(), state.helpers(), user_id, self)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use mockall::predicate::{eq, function};
    use mongodb::bson::oid::ObjectId;
    use validator::Validate;

    use crate::{helpers::Helpers, import_double};

    import_double!(DbClient);

    use super::*;

    #[test]
    fn add_bal_end_req_validate() {
        let mut add_bal_end_req = AddBalEndReq {
            amount: 0,
            transaction_id: ObjectId::new(),
            is_successful: true,
            error_reason: None,
            tracking_id: None,
        };
        let errors = add_bal_end_req.validate().err().unwrap();
        let errors = errors.errors();
        dbg!(&errors);
        assert_eq!(errors.get("amount").is_some(), true);
        add_bal_end_req.amount = 10;
        let result = add_bal_end_req.validate();
        dbg!(&result);
        assert_eq!(result.is_ok(), true);
    }

    #[tokio::test]
    async fn add_bal_end_req_validate_extra() {
        let mut add_bal_end_req = AddBalEndReq {
            amount: 10,
            transaction_id: ObjectId::new(),
            is_successful: false,
            error_reason: None,
            tracking_id: None,
        };
        // test errorReason is required for failed transaction
        let state = Arc::new(AppState::mock());
        let result = add_bal_end_req.validate_extra(state, Some(10)).await;
        dbg!(&result);
        match result {
            Err(AppError::BadRequest(s)) => {
                assert_eq!(s, "errorReason is required for failed transaction")
            }
            _ => panic!(),
        };

        // test validator return Ok
        add_bal_end_req.error_reason = Some("test error reason".to_owned());
        let transaction_id = add_bal_end_req.transaction_id.clone();
        let amount = add_bal_end_req.amount;
        let user_id = 10;
        let mut state = AppState::mock();
        state
            .get_mut_validators()
            .expect_validate_add_bal_transaction()
            .with(
                function(|db: &DbClient| true),
                function(|h: &Helpers| true),
                eq(user_id),
                function(move |body: &AddBalEndReq| {
                    body.amount == amount
                        && !body.is_successful
                        && body.transaction_id == transaction_id
                        && body.error_reason.is_some()
                        && body.tracking_id.is_none()
                }),
            )
            .once()
            .return_once(|_, _, _, _| Ok(()));
        let state = Arc::new(state);
        let result = add_bal_end_req.validate_extra(state, Some(user_id)).await;
        assert_eq!(result.is_ok(), true);

        // test validator return Err
        let transaction_id = add_bal_end_req.transaction_id.clone();
        let mut state = AppState::mock();
        state
            .get_mut_validators()
            .expect_validate_add_bal_transaction()
            .with(
                function(|db: &DbClient| true),
                function(|h: &Helpers| true),
                eq(user_id),
                function(move |body: &AddBalEndReq| {
                    body.amount == amount
                        && !body.is_successful
                        && body.transaction_id == transaction_id
                        && body.error_reason.is_some()
                        && body.tracking_id.is_none()
                }),
            )
            .once()
            .return_once(|_, _, _, _| Err(AppError::BadRequest("Some error".into())));
        let state = Arc::new(state);
        let result = add_bal_end_req.validate_extra(state, Some(user_id)).await;
        match result {
            Err(AppError::BadRequest(s)) => {
                assert_eq!(s, "Some error")
            }
            _ => panic!(),
        };
    }
}
