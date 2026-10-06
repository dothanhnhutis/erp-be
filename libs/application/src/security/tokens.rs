use crate::errors::AppError;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use ctutils::CtEq;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Access token sống ngắn: thu hồi phiên sẽ có hiệu lực chậm nhất sau chừng này
/// (trừ khi bị chặn sớm hơn bằng revoked-cache, xem auth.rs).
pub const ACCESS_TTL: Duration = Duration::minutes(10);

/// Số byte ngẫu nhiên trước khi hex-encode. 32 byte = 256-bit entropy.
const TOKEN_BYTES: usize = 32;

/// Token session vừa được sinh.
/// - `raw`: giá trị THÔ trả cho client (cookie + JSON), 64 ký tự hex (an toàn cho cookie/URL).
/// - `hash`: `hex(sha256(raw))` — 64 ký tự, vừa khít cột `CHAR(64)`; chỉ giá trị này được lưu DB.
pub struct SessionToken {
    pub raw: String,
    pub hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // user id
    pub sid: Uuid, // session id -> biết "phiên hiện tại"
    pub iat: i64,
    pub exp: i64,
}

pub fn issue_access(key: &EncodingKey, user_id: Uuid, sid: Uuid) -> Result<String, AppError> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        sid,
        iat: now.timestamp(),
        exp: (now + ACCESS_TTL).timestamp(),
    };
    encode(&Header::new(Algorithm::HS256), &claims, key)
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub fn verify_access(key: &DecodingKey, token: &str) -> Result<Claims, AppError> {
    let mut v = Validation::new(Algorithm::HS256);
    v.leeway = 30;
    decode::<Claims>(token, key, &v)
        .map(|d| d.claims)
        .map_err(|e| AppError::Unauthorized(e.to_string()))
}

/// Refresh token có dạng "<session_id>.<32 byte ngẫu nhiên base64url>".
/// Tiền tố session_id cho phép tra đúng dòng trong DB bằng primary key,
/// còn phần bí mật chỉ được lưu dưới dạng hash.
pub fn new_refresh() -> SessionToken {
    let mut bytes = [0u8; TOKEN_BYTES];
    rand::rng().fill_bytes(&mut bytes);
    let secret = URL_SAFE_NO_PAD.encode(bytes);
    let hash = hash_secret(&secret);
    SessionToken {
        raw: secret.to_string(),
        hash,
    }
}

pub fn parse_refresh(token: &str) -> Option<(Uuid, &str)> {
    let (sid, secret) = token.split_once('.')?;
    if secret.len() < 40 {
        return None;
    }
    Some((Uuid::parse_str(sid).ok()?, secret))
}

pub fn hash_secret(secret: &str) -> String {
    hex::encode(Sha256::digest(secret.as_bytes()))
}

pub fn hashes_equal(a: &str, b: &str) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).into()
}
