use std::sync::Arc;

#[cfg(test)]
use mockall_double::double;

#[cfg_attr(test, double)]
use super::database::DbClient;

#[cfg_attr(test, double)]
use crate::external_api::ExternalApi;

#[cfg_attr(test, double)]
use crate::validators::Validators;

#[cfg_attr(test, double)]
use crate::utils::Utility;

pub struct AppState {
    db: Arc<DbClient>,
    external_api: Arc<ExternalApi>,
    validators: Arc<Validators>,
    utility: Arc<Utility>,
}

impl AppState {
    pub async fn new() -> Self {
        let db = DbClient::new().await;
        let external_api = ExternalApi::new();
        let utility = Utility::new();
        let validators = Validators::new();
        Self {
            db: Arc::new(db),
            external_api: Arc::new(external_api),
            utility: Arc::new(utility),
            validators: Arc::new(validators),
        }
    }
    pub fn db(&self) -> &DbClient {
        self.db.as_ref()
    }
    pub fn external_api(&self) -> &ExternalApi {
        self.external_api.as_ref()
    }
    pub fn utility(&self) -> &Utility {
        self.utility.as_ref()
    }
    pub fn validators(&self) -> &Validators {
        self.validators.as_ref()
    }
}
