use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::default_route::default_route_handler,
        crate::handlers::ping::ping_handler,
        crate::handlers::ping::temp_api_get_token,
        crate::handlers::ping::temp_api_get_otp,
        crate::handlers::wallet::add_bal::add_bal_init_handler,
        crate::handlers::wallet::add_bal::add_bal_end_handler,

    ),
    components(
        schemas(
            crate::models::AddBalInitReq,
            crate::models::AddBalEndReq,

            crate::models::GenericResponse,
            crate::models::AddBalInitRes,

            crate::models::Money,
            crate::models::LoginScheme,
            crate::models::User,
            crate::models::AdminUser,

        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Debugging API", description = "API for debugging purposes"),
        (name = "App User API", description = "API for app user functionalities"),
        (name = "Admin API", description = "API for admin functionalities")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "authorization",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("authorization"))),
            )
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use std::{collections::HashMap, sync::Arc};

//     use axum::{body::Body, http::Request};
//     use serde::Deserialize;
//     use serde_json::Value as JsonValue;
//     use tower::ServiceExt;

//     use crate::config::build_app_routes;

//     #[mockall_double::double]
//     use crate::database::DbClient;

//     #[derive(Debug, Deserialize)]
//     struct SwaggerInfo {
//         title: String,
//     }

//     #[derive(Debug, Deserialize)]
//     struct SwaggerTag {
//         name: String,
//     }

//     #[derive(Debug, Deserialize)]
//     struct SwaggerJson {
//         info: SwaggerInfo,
//         paths: HashMap<String, JsonValue>,
//         tags: Vec<SwaggerTag>,
//     }

//     const ADMIN_API: &str = "Admin API";
//     const APP_USER_API: &str = "App User API";
//     const DEBUGGING_API: &str = "Debugging API";

//     const ALL_PATHS: [(&str, &str, &str); 2] = [
//         ("/", "get", DEBUGGING_API),
//         ("/api/v1/user/create", "post", APP_USER_API),
//     ];

//     #[tokio::test]
//     async fn test_swagger_api() {
//         let mock = DbClient::default();
//         let mock = Arc::new(mock);
//         let app = build_app_routes(mock);
//         let path = "/api-docs/openapi.json";
//         let req = Request::builder().uri(path).body(Body::empty()).unwrap();
//         let res = app.oneshot(req).await.unwrap();
//         let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
//         let body: SwaggerJson = serde_json::from_slice(&body).unwrap();
//         check_title(&body);
//         check_tags(&body);
//         for (path, method, tag) in ALL_PATHS {
//             check_path(&body, path, method, tag);
//         }
//     }

//     fn check_title(swagger_json: &SwaggerJson) {
//         assert_eq!(swagger_json.info.title, "trailsbuddy_api".to_owned());
//     }

//     fn check_tags(sj: &SwaggerJson) {
//         assert!(sj.tags.iter().any(|t| t.name.as_str() == DEBUGGING_API));
//         assert!(sj.tags.iter().any(|t| t.name.as_str() == APP_USER_API));
//         assert!(sj.tags.iter().any(|t| t.name.as_str() == ADMIN_API));
//     }

//     fn check_path(swagger_json: &SwaggerJson, path: &str, method: &str, tag: &str) {
//         let route = swagger_json.paths.get(path).unwrap().get(method).unwrap();
//         let tags = route.get("tags").unwrap().as_array().unwrap();
//         assert_eq!(tags[0].as_str().unwrap(), tag);
//     }
// }
