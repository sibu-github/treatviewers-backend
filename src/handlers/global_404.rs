use axum::{
    http::{StatusCode, Uri},
    response::IntoResponse,
};

use crate::models::GenericResponse;

/// Global fallback handler
/// Returns a JSON response with 404 status code
pub async fn global_404_handler(uri: Uri) -> impl IntoResponse {
    let msg = format!("Route `{}` does not exist", uri);
    tracing::debug!(msg);
    (
        StatusCode::NOT_FOUND,
        GenericResponse::json_response(false, msg.as_str()),
    )
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
    async fn test_global_404_handler() {
        let app = Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .fallback(global_404_handler);
        let path = "/not-existing-path";
        let req = Request::builder().uri(path).body(Body::empty()).unwrap();
        let res = app.oneshot(req).await.unwrap();
        let status = res.status();
        let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body_str = std::str::from_utf8(&body).unwrap();
        dbg!("body_str {:?}", body_str);
        assert_eq!(status, StatusCode::NOT_FOUND);
        let res: GenericResponse = serde_json::from_slice(&body).unwrap();
        dbg!("{:?}", &res);
        assert_eq!(res.success, false);
        assert_eq!(res.message.contains(path), true);
    }
}
