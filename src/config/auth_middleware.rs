use std::sync::Arc;

use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    http::{Request, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
    TypedHeader,
};

use crate::config::{protected_routes::*, AppError};

use super::AppState;

pub async fn auth_middleware<B>(
    uri: Uri,
    bearer: Option<TypedHeader<Authorization<Bearer>>>,
    State(state): State<Arc<AppState>>,
    mut request: Request<B>,
    next: Next<B>,
) -> Response {
    tracing::debug!("Running auth_middleware: path = {}", uri.path());
    match bearer {
        Some(bearer) => match state.utility().decode_token(bearer.token()) {
            Err(e) => {
                if is_admin_only_path(&uri) || !is_unprotected_path(&uri) {
                    return e.into_response();
                }
            }
            Ok(claims) => {
                if is_admin_only_path(&uri) {
                    if !claims.is_admin {
                        let err = "Unauthorized for ADMIN ONLY path";
                        return AppError::Auth(err.into()).into_response();
                    }
                    // TODO: implement extra check to validate admin user from database
                }
                request.extensions_mut().insert(claims);
            }
        },
        None => {
            if is_admin_only_path(&uri) || !is_unprotected_path(&uri) {
                return AppError::Auth("missing token".into()).into_response();
            }
        }
    };

    let response = next.run(request).await;
    response
}

#[cfg(test)]
mod tests {

    use axum::{body::Body, http::StatusCode, middleware, routing::get, Extension, Json, Router};
    use mockall::predicate::eq;

    use crate::{
        models::*,
        utils::{get_epoch_ts, test_helper::*},
    };

    use super::*;

    async fn handler_with_claim_extractor(
        Extension(_claims): Extension<JwtClaims>,
        State(_state): State<Arc<AppState>>,
    ) -> Json<GenericResponse> {
        GenericResponse::json_response(true, "")
    }

    async fn handler_without_claim_extractor(
        State(_state): State<Arc<AppState>>,
    ) -> Json<GenericResponse> {
        GenericResponse::json_response(true, "")
    }

    fn create_app(state: Arc<AppState>) -> Router<(), Body> {
        let unprotected_path = get_an_unprotected_path();
        let admin_path = get_an_admin_path();
        let protected_path = "/protected_path";
        Router::new()
            .route(
                protected_path,
                get(handler_with_claim_extractor).post(handler_without_claim_extractor),
            )
            .route(
                admin_path,
                get(handler_without_claim_extractor).post(handler_with_claim_extractor),
            )
            .route(
                unprotected_path,
                get(handler_without_claim_extractor).post(handler_with_claim_extractor),
            )
            .route_layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_auth_middleware_unprotected_path() {
        // unprotected path no token passed
        let unprotected_path = get_an_unprotected_path();
        let state = Arc::new(AppState::mock());
        let app = create_app(state);
        let body = build_get_request(unprotected_path, None);
        let res: GenericResponse = oneshot_request(app, body, Some(StatusCode::OK)).await;
        assert_eq!(res.success, true);

        // unprotected path token passed and claim extractor in the handler
        let ts = get_epoch_ts();
        let mut state = AppState::mock();
        state
            .get_mut_utility()
            .expect_decode_token()
            .once()
            .with(eq("dummy_token"))
            .returning(move |_| {
                Ok(JwtClaims {
                    id: 5,
                    name: None,
                    is_admin: false,
                    exp: ts as usize,
                })
            });
        let state = Arc::new(state);
        let app = create_app(state);
        let body = build_post_request(unprotected_path, "", Some("dummy_token"));
        let res: GenericResponse = oneshot_request(app, body, Some(StatusCode::OK)).await;
        assert_eq!(res.success, true);
    }

    #[tokio::test]
    async fn test_auth_middleware_unprotected_path_no_token_claim_extractor() {
        // unprotected path no token passed but claim extractor in the handler
        let unprotected_path = get_an_unprotected_path();
        let state = Arc::new(AppState::mock());
        let app = create_app(state);
        let body = build_post_request(unprotected_path, "", None);
        let res = oneshot_req_plain(app, body, Some(StatusCode::INTERNAL_SERVER_ERROR)).await;
        assert_eq!(res.contains("Missing request extension"), true);
    }

    #[tokio::test]
    async fn test_auth_middleware_protected_path() {
        // protected path token passed and claim extractor in the handler
        let protected_path = "/protected_path";
        let ts = get_epoch_ts();
        let mut state = AppState::mock();
        state
            .get_mut_utility()
            .expect_decode_token()
            .once()
            .with(eq("dummy_token"))
            .returning(move |_| {
                Ok(JwtClaims {
                    id: 5,
                    name: None,
                    is_admin: false,
                    exp: ts as usize,
                })
            });
        let state = Arc::new(state);
        let app = create_app(state);
        let body = build_post_request(protected_path, "", Some("dummy_token"));
        let res: GenericResponse = oneshot_request(app, body, Some(StatusCode::OK)).await;
        assert_eq!(res.success, true);

        // protected path no token passed should be Auth error
        let mut state = AppState::mock();
        state.get_mut_utility().expect_decode_token().never();
        let state = Arc::new(state);
        let app = create_app(state);
        let body = build_post_request(protected_path, "", None);
        let res: GenericResponse = oneshot_request(app, body, Some(StatusCode::UNAUTHORIZED)).await;
        assert_eq!(res.success, false);
        assert_eq!(res.message, "missing token");

        // protected path invalid token passed should be Auth error
        let mut state = AppState::mock();
        state
            .get_mut_utility()
            .expect_decode_token()
            .once()
            .with(eq("dummy_token"))
            .returning(|_| Err(AppError::Auth("Invalid token".into())));
        let state = Arc::new(state);
        let app = create_app(state);
        let body = build_post_request(protected_path, "", Some("dummy_token"));
        let res: GenericResponse = oneshot_request(app, body, Some(StatusCode::UNAUTHORIZED)).await;
        assert_eq!(res.success, false);
        assert_eq!(res.message, "Invalid token");
    }

    #[tokio::test]
    async fn test_auth_middleware_admin_path() {
        let admin_path = get_an_admin_path();
        let ts = get_epoch_ts();
        // admin path no token should be auth error
        let mut state = AppState::mock();
        state.get_mut_utility().expect_decode_token().never();
        let state = Arc::new(state);
        let app = create_app(state);
        let body = build_get_request(admin_path, None);
        let res: GenericResponse = oneshot_request(app, body, Some(StatusCode::UNAUTHORIZED)).await;
        assert_eq!(res.success, false);
        assert_eq!(res.message, "missing token");

        // admin path with invalid token should be auth error
        let mut state = AppState::mock();
        state
            .get_mut_utility()
            .expect_decode_token()
            .once()
            .with(eq("dummy_token"))
            .returning(|_| Err(AppError::Auth("Invalid token".into())));
        let state = Arc::new(state);
        let app = create_app(state);
        let body = build_get_request(admin_path, Some("dummy_token"));
        let res: GenericResponse = oneshot_request(app, body, Some(StatusCode::UNAUTHORIZED)).await;
        assert_eq!(res.success, false);
        assert_eq!(res.message, "Invalid token");

        // admin path valid token but not admin token
        let mut state = AppState::mock();
        state
            .get_mut_utility()
            .expect_decode_token()
            .once()
            .with(eq("dummy_token"))
            .returning(move |_| {
                Ok(JwtClaims {
                    id: 5,
                    name: None,
                    is_admin: false,
                    exp: ts as usize,
                })
            });
        let state = Arc::new(state);
        let app = create_app(state);
        let body = build_get_request(admin_path, Some("dummy_token"));
        let res: GenericResponse = oneshot_request(app, body, Some(StatusCode::UNAUTHORIZED)).await;
        assert_eq!(res.success, false);
        assert_eq!(res.message, "Unauthorized for ADMIN ONLY path");

        // admin path valid token & claim extracted in handler
        let mut state = AppState::mock();
        state
            .get_mut_utility()
            .expect_decode_token()
            .once()
            .with(eq("dummy_token"))
            .returning(move |_| {
                Ok(JwtClaims {
                    id: 5,
                    name: None,
                    is_admin: true,
                    exp: ts as usize,
                })
            });
        let state = Arc::new(state);
        let app = create_app(state);
        let body = build_post_request(admin_path, "", Some("dummy_token"));
        let res: GenericResponse = oneshot_request(app, body, Some(StatusCode::OK)).await;
        assert_eq!(res.success, true);
    }
}
