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
    Validation(#[from] ValidationErrors), // input: fail rule validator (email, length...)
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
            _ => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "errors": "lỗi chưa được xử lý" })),
            )
                .into_response(),
        }
    }
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
