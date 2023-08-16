use axum::Json;

use crate::models::GenericResponse;

/// Default endpoint
///
/// Returns a JSON response with 200 status code
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Get success response from server", body=GenericResponse)
    ),
    tag = "Debugging API"
)]
pub async fn default_route_handler() -> Json<GenericResponse> {
    GenericResponse::json_response(true, "Server is running")
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn test_default_route_handler() {
        let app = Router::new().route("/", get(default_route_handler));
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = app.oneshot(req).await.unwrap();
        let status = res.status();
        let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body_str = std::str::from_utf8(&body).unwrap();
        dbg!("body_str {:?}", body_str);
        assert_eq!(status, StatusCode::OK);
        let res: GenericResponse = serde_json::from_slice(&body).unwrap();
        dbg!("{:?}", &res);
        assert_eq!(res.success, true);
        assert_eq!(res.message, "Server is running".to_owned());
    }
}
