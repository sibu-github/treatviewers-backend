use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::models::GenericResponse;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    Auth(String),
    AnyError(anyhow::Error),
}

impl AppError {
    pub fn unknown_error() -> Self {
        Self::AnyError(anyhow::anyhow!("Unknown error"))
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(err: E) -> Self {
        Self::AnyError(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest(msg) => {
                tracing::debug!("Bad request: {}", msg);
                let res = GenericResponse::json_response(false, msg.as_str());
                (StatusCode::BAD_REQUEST, res).into_response()
            }
            Self::NotFound(msg) => {
                tracing::debug!("Not Found: {}", msg);
                let res = GenericResponse::json_response(false, msg.as_str());
                (StatusCode::NOT_FOUND, res).into_response()
            }
            Self::Auth(msg) => {
                tracing::debug!("Unauthorized: {}", msg);
                let res = GenericResponse::json_response(false, msg.as_str());
                (StatusCode::UNAUTHORIZED, res).into_response()
            }
            Self::AnyError(err) => {
                let msg = format!("Something went wrong: {err}");
                tracing::debug!("{msg}");
                let res = GenericResponse::json_response(false, msg.as_str());
                (StatusCode::INTERNAL_SERVER_ERROR, res).into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    async fn check_response(status_code: StatusCode, msg: &str, app_error: AppError) {
        let res = app_error.into_response();
        let status = res.status();
        let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body_str = std::str::from_utf8(&body).unwrap();
        dbg!("body_str {:?}", body_str);
        let res: GenericResponse = serde_json::from_slice(&body).unwrap();
        dbg!("{:?}", &res);
        assert_eq!(status, status_code);
        assert_eq!(res.success, false);
        assert_eq!(res.message, msg.to_owned());
    }

    #[tokio::test]
    async fn test_app_error_bad_request() {
        let msg = "Bad Request error message";
        let app_error = AppError::BadRequest(msg.to_owned());
        check_response(StatusCode::BAD_REQUEST, msg, app_error).await;
    }

    #[tokio::test]
    async fn test_app_error_not_found() {
        let msg = "Not found error message";
        let app_error = AppError::NotFound(msg.to_owned());
        check_response(StatusCode::NOT_FOUND, msg, app_error).await;
    }

    #[tokio::test]
    async fn test_app_error_auth() {
        let msg = "Auth error message";
        let app_error = AppError::Auth(msg.to_owned());
        check_response(StatusCode::UNAUTHORIZED, msg, app_error).await;
    }

    #[tokio::test]
    async fn test_app_error_any() {
        let msg = "anyhow error message";
        let err = anyhow::anyhow!(msg);
        let app_error = AppError::AnyError(err);
        let msg = format!("Something went wrong: {msg}");
        check_response(StatusCode::INTERNAL_SERVER_ERROR, &msg, app_error).await;
    }

    #[tokio::test]
    async fn test_app_error_unknown_error() {
        let app_error = AppError::unknown_error();
        let msg = format!("Something went wrong: Unknown error");
        check_response(StatusCode::INTERNAL_SERVER_ERROR, &msg, app_error).await;
    }
}
