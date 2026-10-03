use rand::Rng;
use sha2::{Digest, Sha256};

/// Số byte ngẫu nhiên trước khi hex-encode. 32 byte = 256-bit entropy.
const TOKEN_BYTES: usize = 32;

/// Token session vừa được sinh.
/// - `raw`: giá trị THÔ trả cho client (cookie + JSON), 64 ký tự hex (an toàn cho cookie/URL).
/// - `hash`: `hex(sha256(raw))` — 64 ký tự, vừa khít cột `CHAR(64)`; chỉ giá trị này được lưu DB.
pub struct SessionToken {
    pub raw: String,
    pub hash: String,
}

impl SessionToken {
    pub fn generate() -> Self {
        let mut bytes = [0u8; TOKEN_BYTES];
        rand::rng().fill_bytes(&mut bytes);
        let raw = hex::encode(bytes);
        let hash = hash_token(&raw);
        Self { raw, hash }
    }
}

/// `hex(sha256(token))` — dùng chung lúc tạo và (sau này) lúc xác thực session.
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
