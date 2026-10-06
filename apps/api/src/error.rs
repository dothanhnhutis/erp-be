use application::errors::AppError;
use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    JsonRejection(#[from] JsonRejection), // input: body hỏng / sai content-type / sai kiểu

    #[error(transparent)]
    Validation(#[from] ValidationErrors),

    #[error(transparent)]
    Domain(#[from] AppError), // nghiệp vụ bubble từ use case lên // input: fail rule validator (email, length...)
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            // JsonRejection tự biết status: 400 syntax / 415 content-type / 422 sai kiểu
            ApiError::JsonRejection(rejection) => {
                let status = rejection.status();
                (status, Json(json!({ "error": rejection.body_text() }))).into_response()
            }
            // field-level errors → FE map được vào từng input (kiểu Zod)
            ApiError::Validation(errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({ "errors": fields_to_json(&errors) })),
            )
                .into_response(),
            ApiError::Domain(err) => domain_to_response(err),
        }
    }
}

fn domain_to_response(err: AppError) -> Response {
    let (status, msg) = match err {
        AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        AppError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
        AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
        AppError::Internal(msg) => {
            tracing::error!("Internal error: {msg}"); // log đầy đủ phía server
            (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi hệ thống".to_owned()) // client nhận câu chung
        }
        AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),

        AppError::Conflict(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
    };
    (status, Json(json!({ "error": msg }))).into_response()
}

fn fields_to_json(errors: &ValidationErrors) -> serde_json::Value {
    let map: serde_json::Map<_, _> = errors
        .field_errors()
        .iter()
        .map(|(field, errs)| {
            let msgs: Vec<String> = errs
                .iter()
                .map(|e| {
                    e.message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| format!("{field} không hợp lệ"))
                })
                .collect();
            (field.to_string(), json!(msgs))
        })
        .collect();
    json!(map)
}
