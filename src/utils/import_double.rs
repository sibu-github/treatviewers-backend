#[macro_export]
macro_rules! import_double {
    (DbClient) => {
        #[cfg_attr(test, mockall_double::double)]
        use crate::config::database::DbClient;
    };
    (DbSession) => {
        #[cfg_attr(test, mockall_double::double)]
        use crate::config::database_session::DbSession;
    };
    (ExternalApi) => {
        #[cfg_attr(test, mockall_double::double)]
        use crate::external_api::ExternalApi;
    };
    (Validators) => {
        #[cfg_attr(test, mockall_double::double)]
        use crate::validators::Validators;
    };
    (Utility) => {
        #[cfg_attr(test, mockall_double::double)]
        use crate::utils::Utility;
    };
    ($item:ident, $($more:ident),+) => {
        import_double!($item);
        import_double!($($more),+);
    };
    ($item: ty) => {
        #[cfg_attr(test, mockall_double::double)]
        use $item;
    };
    ($item:ty, $($more:ty),+) => {
        use $item;
        import_double!($($more),+);
    };
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_import_db_client() {
        import_double!(DbClient);
        fn test_macro_import(db: &DbClient) {}
    }
    #[test]
    fn test_import_db_session() {
        import_double!(DbSession);
        fn test_macro_import(db: &DbSession) {}
    }
    #[test]
    fn test_import_external_api() {
        import_double!(ExternalApi);
        fn test_macro_import(api: &ExternalApi) {}
    }
    #[test]
    fn test_import_validators() {
        import_double!(Validators);
        fn test_macro_import(api: &Validators) {}
    }
    #[test]
    fn test_import_utility() {
        import_double!(Utility);
        fn test_macro_import(util: &Utility) {}
    }
    #[test]
    fn test_import_multiple_iden() {
        import_double!(DbClient, DbSession, ExternalApi, Validators, Utility);
        fn test_macro_import(
            db: &DbClient,
            session: &DbSession,
            api: &ExternalApi,
            v: &Validators,
            util: &Utility,
        ) {
        }
    }
    #[test]
    fn test_import_by_path() {
        import_double!(crate::config::database::DbClient);
        fn test_macro_import(db: &DbClient) {}
    }
    #[test]
    fn test_import_multiple_path() {
        import_double!(
            crate::config::database::DbClient,
            crate::config::database_session::DbSession
        );
        fn test_macro_import1(db: &DbClient) {}
        fn test_macro_import2(db: &DbSession) {}
    }
}
