use serde::{Deserialize, Serialize};
use validator::Validate;

/// Admin tạo user mới: email + danh sách role.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct CreateUserRequest {
    #[validate(
        email(message = "Email không hợp lệ"),
        length(max = 255, message = "Email không hợp lệ")
    )]
    pub email: String,
    pub role_ids: Vec<uuid::Uuid>,
}

#[derive(Debug, Serialize)]
pub struct CreateUserResponse {
    pub user_id: String,
}
