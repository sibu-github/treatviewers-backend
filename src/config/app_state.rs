use crate::import_double;
use std::sync::Arc;

use crate::helpers::Helpers;

import_double!(DbClient, ExternalApi, Validators, Utility);

pub struct AppState {
    db: Arc<DbClient>,
    external_api: Arc<ExternalApi>,
    validators: Arc<Validators>,
    utility: Arc<Utility>,
    helpers: Arc<Helpers>,
}

impl AppState {
    pub async fn new() -> Self {
        let db = DbClient::new().await;
        let external_api = ExternalApi::new();
        let utility = Utility::new();
        let validators = Validators::new();
        let helpers = Helpers::new();
        Self {
            db: Arc::new(db),
            external_api: Arc::new(external_api),
            utility: Arc::new(utility),
            validators: Arc::new(validators),
            helpers: Arc::new(helpers),
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
    pub fn helpers(&self) -> &Helpers {
        self.helpers.as_ref()
    }
    #[cfg(test)]
    pub fn mock() -> Self {
        let db = Arc::new(DbClient::default());
        let external_api = Arc::new(ExternalApi::default());
        let validators = Arc::new(Validators::default());
        let utility = Arc::new(Utility::default());
        let helpers = Arc::new(Helpers::mock());
        Self {
            db,
            external_api,
            validators,
            utility,
            helpers,
        }
    }
    #[cfg(test)]
    pub fn get_mut_db(&mut self) -> &mut DbClient {
        Arc::get_mut(&mut self.db).unwrap()
    }
    #[cfg(test)]
    pub fn get_mut_external_api(&mut self) -> &mut ExternalApi {
        Arc::get_mut(&mut self.external_api).unwrap()
    }
    #[cfg(test)]
    pub fn get_mut_validators(&mut self) -> &mut Validators {
        Arc::get_mut(&mut self.validators).unwrap()
    }
    #[cfg(test)]
    pub fn get_mut_utility(&mut self) -> &mut Utility {
        Arc::get_mut(&mut self.utility).unwrap()
    }
    #[cfg(test)]
    pub fn get_mut_helpers(&mut self) -> &mut Helpers {
        Arc::get_mut(&mut self.helpers).unwrap()
    }

    #[cfg(test)]
    pub fn mock_app_state(
        db: DbClient,
        external_api: ExternalApi,
        validators: Validators,
        utility: Utility,
        helpers: Helpers,
    ) -> Self {
        let db = Arc::new(db);
        let external_api = Arc::new(external_api);
        let validators = Arc::new(validators);
        let utility = Arc::new(utility);
        let helpers = Arc::new(helpers);
        Self {
            db,
            external_api,
            validators,
            utility,
            helpers,
        }
    }
}
