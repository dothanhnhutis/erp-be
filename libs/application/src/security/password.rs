use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};

use crate::errors::AppError;

/// Hash mật khẩu mới (argon2id, salt ngẫu nhiên) → chuỗi PHC để lưu DB.
pub fn hash_password(plain: &str) -> Result<String, AppError> {
    Argon2::default()
        .hash_password(plain.as_bytes())
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(format!("Lỗi hash mật khẩu: {e}")))
}

/// Kiểm tra mật khẩu so với chuỗi PHC đã lưu.
pub fn verify_password(plain: &str, phc: &str) -> bool {
    match PasswordHash::new(phc) {
        Ok(parsed) => Argon2::default()
            .verify_password(plain.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}
