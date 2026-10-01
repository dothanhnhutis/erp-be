use crate::extractors::validate::ValidatedBodyJson;
use axum::response::IntoResponse;
use serde::Deserialize;
use validator::{Validate, ValidationError};

#[derive(Debug, Validate, Deserialize)]
pub struct LoginPayload {
    #[validate(email)]
    email: String,
    #[validate(
        length(min = 1, message = "Email và mật khẩu không hợp lệ."),
        custom(function = "validate_password")
    )]
    pub password: String,
    pub app_version: Option<String>,
    pub platform: Option<String>,
    #[validate(custom(function = "validate_device_type"))]
    pub device_type: String,
    pub device_name: Option<String>,
    pub device_id: Option<String>,
}

fn validate_password(password: &str) -> Result<(), ValidationError> {
    let mut has_uppercase = false;
    let mut has_lowercase = false;
    let mut has_digit = false;

    for c in password.chars() {
        has_uppercase |= c.is_ascii_uppercase();
        has_lowercase |= c.is_ascii_lowercase();
        has_digit |= c.is_ascii_digit();
    }
    if !password.is_empty() && has_uppercase && has_lowercase && has_digit {
        Ok(())
    } else {
        Err(ValidationError {
            code: "invalid_password".into(),
            message: Some("Email và mật khẩu không hợp lệ.".into()),
            params: Default::default(),
        })
    }
}

fn validate_device_type(device_type: &str) -> Result<(), ValidationError> {
    let device_types = ["web", "mobile", "desktop"];

    if device_types.contains(&device_type) {
        Ok(())
    } else {
        let mut error = ValidationError::new("invalid_device_type");
        error.message = Some("Loại thiết bị không hợp lệ.".into());
        Err(error)
    }
}

pub async fn login_handler(
    ValidatedBodyJson(payload): ValidatedBodyJson<LoginPayload>,
) -> impl IntoResponse {
    println!("{:#?}", payload);

    "Ok"
}
