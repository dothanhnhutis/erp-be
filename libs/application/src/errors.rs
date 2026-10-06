use domain::{errors::DomainError, repositories::RepositoryError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict error: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<DomainError> for AppError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::InvalidEmail(m)
            | DomainError::InvalidUserStatus(m)
            | DomainError::InvalidDeviceType(m)
            | DomainError::InvalidRoleStatus(m)
            | DomainError::InvalidPasswordTokenType(m)
            | DomainError::RevokedPasswordToken(m) => AppError::Validation(m),
            DomainError::Conflict(m) => AppError::Conflict(m),
        }
    }
}

impl From<RepositoryError> for AppError {
    fn from(e: RepositoryError) -> Self {
        match e {
            RepositoryError::NotFound => AppError::NotFound("Không tìm thấy".into()),
            RepositoryError::UniqueViolation(c) => {
                let msg = match c.as_str() {
                    "users_email_key" => "Email đã được sử dụng",
                    "users_username_key" => "Username đã được sử dụng",
                    "user_sessions_token_hash_key" => "Session token đã tồn tại",
                    _ => "Dữ liệu đã tồn tại",
                };
                AppError::Conflict(msg.into())
            }
            RepositoryError::ForeignKeyViolation(_) => {
                AppError::Validation("Tham chiếu không hợp lệ".into())
            }
            // DB chứa dữ liệu domain coi là sai → lỗi hệ thống, KHÔNG phải lỗi client
            RepositoryError::Mapping(d) => AppError::Internal(d.to_string()),
            RepositoryError::Unexpected(err) => AppError::Internal(err.to_string()),
        }
    }
}

// impl From<StorageError> for AppError {
//     fn from(e: StorageError) -> Self {
//         match e {
//             StorageError::Unavailable(err) => AppError::Internal(err.to_string()),
//             StorageError::Unauthorized => AppError::Internal(String::new()),
//             StorageError::NotFound { key } => AppError::Internal(key.to_string()),
//             StorageError::InvalidRequest(err) => AppError::Internal(err.to_string()),
//             StorageError::Internal(err) => AppError::Internal(err.to_string()),
//         }
//     }
// }
