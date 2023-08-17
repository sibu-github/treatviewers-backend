use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde::de::DeserializeOwned;
use tower::ServiceExt;

#[cfg(test)]
pub fn build_post_request(path: &str, body: &str, token: Option<&str>) -> Request<Body> {
    let builder = Request::builder()
        .uri(path)
        .method("POST")
        .header("Content-Type", "application/json");
    let builder = if let Some(token) = token {
        builder.header("Authorization", format!("Bearer {token}"))
    } else {
        builder
    };
    builder.body(Body::from(body.to_owned())).unwrap()
}

#[cfg(test)]
pub fn build_get_request(path: &str, token: Option<&str>) -> Request<Body> {
    let builder = Request::builder().uri(path);
    let builder = if let Some(token) = token {
        builder.header("Authorization", format!("Bearer {token}"))
    } else {
        builder
    };
    builder.body(Body::empty()).unwrap()
}

#[cfg(test)]
pub async fn oneshot_request<T>(
    app: Router,
    body: Request<Body>,
    expected_status: Option<StatusCode>,
) -> T
where
    T: DeserializeOwned,
{
    let uri = body.uri().to_string();
    let res = app.oneshot(body).await.unwrap();
    let status = res.status();
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let body = std::str::from_utf8(&body).unwrap();
    dbg!("response for {} -> {}", uri, body);
    if let Some(expected_status) = expected_status {
        assert_eq!(status, expected_status);
    }
    let body: T = serde_json::from_str(body).unwrap();
    body
}

#[cfg(test)]
pub async fn oneshot_req_plain(
    app: Router,
    body: Request<Body>,
    expected_status: Option<StatusCode>,
) -> String {
    let uri = body.uri().to_string();
    let res = app.oneshot(body).await.unwrap();
    let status = res.status();
    let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
    let body = std::str::from_utf8(&body).unwrap();
    dbg!("response for {} -> {}", uri, body);
    body.to_owned()
}
