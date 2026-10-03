use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::error::AppError;

/// Access token sống ngắn: thu hồi phiên sẽ có hiệu lực chậm nhất sau chừng này
/// (trừ khi bị chặn sớm hơn bằng revoked-cache, xem auth.rs).
pub const ACCESS_TTL: Duration = Duration::minutes(10);

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
    encode(&Header::new(Algorithm::HS256), &claims, key).map_err(|e| AppError::Internal(e.to_string()))
}

pub fn verify_access(key: &DecodingKey, token: &str) -> Result<Claims, AppError> {
    let mut v = Validation::new(Algorithm::HS256);
    v.leeway = 30;
    decode::<Claims>(token, key, &v)
        .map(|d| d.claims)
        .map_err(|_| AppError::Unauthorized)
}

/// Refresh token có dạng "<session_id>.<32 byte ngẫu nhiên base64url>".
/// Tiền tố session_id cho phép tra đúng dòng trong DB bằng primary key,
/// còn phần bí mật chỉ được lưu dưới dạng hash.
pub fn new_refresh(sid: Uuid) -> (String, String) {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let secret = URL_SAFE_NO_PAD.encode(bytes);
    let hash = hash_secret(&secret);
    (format!("{sid}.{secret}"), hash)
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
