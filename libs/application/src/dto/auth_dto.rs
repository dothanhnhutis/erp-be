use serde::{Deserialize, Serialize};
use validator::Validate;

/// Metadata lấy từ tầng HTTP (không nằm trong body request).
#[derive(Debug, Default, Clone)]
pub struct ClientContext {
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, message = "Email và mật khẩu không hợp lệ."))]
    pub password: String,
    pub app_version: Option<String>,
    pub platform: Option<String>,
    pub device_type: String,
    pub device_name: Option<String>,
    pub device_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user_id: String,
    /// Token session THÔ — desktop dùng làm bearer, web app lưu trong cookie.
    pub session: String,
    /// Thời gian sống của session, tính bằng giây.
    pub expires_in: i64,
}
