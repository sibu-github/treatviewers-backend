use std::{sync::Arc, time::Duration};

use axum::{
    body::boxed,
    http::{header, HeaderValue},
    routing::get,
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer, set_header::SetResponseHeaderLayer, timeout::TimeoutLayer, trace::TraceLayer,
    ServiceBuilderExt,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{constants::*, handlers::*};

use super::{api_routes::api_routes, swagger::ApiDoc, AppState};

pub fn build_app_routes(state: Arc<AppState>) -> Router {
    let server_header_value = HeaderValue::from_static("trailsbuddy_api");
    let set_response_header_layer =
        SetResponseHeaderLayer::if_not_present(header::SERVER, server_header_value);
    let trace_layer = TraceLayer::new_for_http();
    let cors_layer = CorsLayer::permissive();
    let timeout_layer = TimeoutLayer::new(Duration::from_secs(REQUEST_TIMEOUT_SECS));
    let middleware = ServiceBuilder::new()
        .layer(timeout_layer)
        .layer(cors_layer)
        .layer(set_response_header_layer)
        .map_response_body(boxed)
        .compression()
        .trim_trailing_slash()
        .catch_panic()
        .layer(trace_layer)
        .into_inner();

    let router = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/", get(default_route_handler))
        .nest("/api/v1", api_routes(state.clone()));

    // add /timeout path for testing timeout layer
    #[cfg(test)]
    let router = router.route("/timeout", get(timeout_route_handler));

    // add /catch-panic path for testing catch_panic layer
    #[cfg(test)]
    let router = router.route("/catch-panic", get(catch_panic_handler));

    router
        .layer(middleware)
        .fallback(global_404_handler)
        .with_state(state)
}

// #[cfg(test)]
// mod tests {
//     use axum::{
//         body::Body,
//         http::{header::ACCEPT_ENCODING, Request, StatusCode},
//         Router,
//     };
//     use tower::{Service, ServiceExt};

//     use super::*;

//     fn build_mock_app() -> Router {
//         let mock_app_state = DbClient::default();
//         let mock_app_state = Arc::new(mock_app_state);
//         let app = build_app_routes(mock_app_state);
//         app
//     }

//     #[tokio::test]
//     #[ignore]
//     async fn test_build_app_routes_timeout_layer() {
//         let app = build_mock_app();
//         let req = Request::builder()
//             .uri("/timeout")
//             .body(Body::empty())
//             .unwrap();
//         let res = app.oneshot(req).await.unwrap();
//         let status = res.status();
//         let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
//         let body_str = std::str::from_utf8(&body).unwrap();
//         dbg!("body_str {:?}", body_str);
//         assert_eq!(status, StatusCode::REQUEST_TIMEOUT);
//     }

//     #[tokio::test]
//     async fn test_build_app_routes_catch_panic_layer() {
//         let mut app = build_mock_app();
//         let req = Request::builder()
//             .uri("/catch-panic")
//             .body(Body::empty())
//             .unwrap();
//         let res = app.call(req).await.unwrap();
//         let status = res.status();
//         let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
//         let body_str = std::str::from_utf8(&body).unwrap();
//         dbg!("body_str {:?}", body_str);
//         assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
//         // send another request after catch panic
//         let req = Request::builder().uri("/").body(Body::empty()).unwrap();
//         let res = app.call(req).await.unwrap();
//         let status = res.status();
//         let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
//         let body_str = std::str::from_utf8(&body).unwrap();
//         dbg!("body_str {:?}", body_str);
//         assert_eq!(status, StatusCode::OK);
//     }

//     #[tokio::test]
//     async fn test_build_app_routes_default_route_exists() {
//         let app = build_mock_app();
//         let req = Request::builder()
//             .uri("/")
//             .header(ACCEPT_ENCODING, "gzip")
//             .body(Body::empty())
//             .unwrap();
//         let res = app.oneshot(req).await.unwrap();
//         let status = res.status();
//         let headers = res.headers().to_owned();
//         assert_eq!(status, StatusCode::OK);
//         dbg!("{:?}", &headers);
//         // compression layer
//         assert_eq!(headers["content-encoding"], "gzip");
//         // SetResponseHeaderLayer
//         assert_eq!(headers["server"], "trailsbuddy_api");
//         // CorsLayer
//         assert_eq!(headers["access-control-allow-origin"], "*");
//     }

//     #[tokio::test]
//     async fn test_global_fallback_exists() {
//         let app = build_mock_app();
//         let path = "/not-existing-path";
//         let req = Request::builder().uri(path).body(Body::empty()).unwrap();
//         let res = app.oneshot(req).await.unwrap();
//         let status = res.status();
//         let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
//         let body_str = std::str::from_utf8(&body).unwrap();
//         dbg!("body_str {:?}", body_str);
//         assert_eq!(status, StatusCode::NOT_FOUND);
//     }

//     #[tokio::test]
//     async fn test_swagger_docs_exists() {
//         let mut app = build_mock_app();
//         let path = "/api-docs/openapi.json";
//         let req = Request::builder().uri(path).body(Body::empty()).unwrap();
//         let res = app.call(req).await.unwrap();
//         let status = res.status();
//         let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
//         let body_str = std::str::from_utf8(&body).unwrap();
//         dbg!("body_str {:?}", body_str);
//         assert_eq!(status, StatusCode::OK);
//         let path = "/docs";
//         let req = Request::builder().uri(path).body(Body::empty()).unwrap();
//         let res = app.call(req).await.unwrap();
//         let status = res.status();
//         let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
//         let body_str = std::str::from_utf8(&body).unwrap();
//         dbg!("body_str {:?}", body_str);
//         dbg!("status {:?}", status);
//         assert_eq!(status.is_success() || status.is_redirection(), true);
//     }
// }
