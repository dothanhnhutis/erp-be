use std::sync::LazyLock;

use crate::{
    dto::auth::{ClientContext, LoginRequest, LoginResponse},
    error::AppError,
    security::{
        password::{hash_password, verify_password},
        tokens::{Claims, issue_access, new_refresh},
    },
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Duration;
use domain::{
    entities::{session::NewSession, user::UserStatus},
    repositories::{SessionRepo, UserRepo},
};
use jsonwebtoken::EncodingKey;
use rand::Rng;

static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    hash_password(&URL_SAFE_NO_PAD.encode(bytes)).expect("hash dummy password")
});

pub struct AuthService<UR, SR>
where
    UR: UserRepo,
    SR: SessionRepo,
{
    user_repo: UR,
    session_repo: SR,
}

impl<UR, SR> AuthService<UR, SR>
where
    UR: UserRepo,
    SR: SessionRepo,
{
    pub fn new(user_repo: UR, session_repo: SR) -> Self {
        LazyLock::force(&DUMMY_HASH);
        Self {
            user_repo,
            session_repo,
        }
    }

    pub async fn login(
        &self,
        payload: LoginRequest,
        client_context: ClientContext,
        access_token_ttl: i64,
        refresh_ttl: i64,
        max_refresh_ttl: i64,
        jwt_enc: EncodingKey,
    ) -> Result<LoginResponse, AppError> {
        let device_type = payload.device_type()?;

        let user = self
            .user_repo
            .find_by_email(&payload.email.to_lowercase())
            .await?;

        // lấy mật khẩu đã hash của user. Nếu user chưa đặt mật khẩu
        // thì lấy mật khẩu DUMMY_HASH để tránh lộ bước xác thực
        let hash_str = user
            .as_ref()
            .and_then(|u| u.password_hash.clone())
            .unwrap_or_else(|| DUMMY_HASH.clone());

        let ok = tokio::task::spawn_blocking(move || verify_password(&payload.password, &hash_str))
            .await
            .map_err(|e| AppError::Internal(format!("password verify task failed: {e}")))?;

        let user = match user {
            Some(u) if ok => u,
            _ => {
                return Err(AppError::Unauthorized(
                    "Email hoặc mật khẩu không đúng".into(),
                ));
            }
        };

        match user.status {
            UserStatus::Active => {}
            UserStatus::Deactivated => {
                return Err(AppError::Unauthorized("Tài khoản đã bị vô hiệu hóa".into()));
            }
            UserStatus::PendingPassword => {
                return Err(AppError::Unauthorized(
                    "Email hoặc mật khẩu không đúng".into(),
                ));
            }
        }

        let refresh_token = new_refresh();
        let now = chrono::Utc::now();
        let access_token_ttl = now + Duration::seconds(access_token_ttl);
        let expires_at = now + Duration::seconds(refresh_ttl);
        let absolute_expires_at = now + Duration::seconds(max_refresh_ttl);
        let new_session = NewSession {
            user_id: user.id,
            refresh_token_hash: refresh_token.hash,
            device_type,
            device_name: payload.device_name,
            app_version: payload.app_version,
            user_agent: client_context.user_agent,
            ip_address: client_context.ip_address,
            expires_at,
            absolute_expires_at,
        };

        let new_session = self.session_repo.create(new_session).await?;

        let claims = Claims {
            sub: user.id,
            sid: new_session.id,
            iat: now.timestamp(),
            exp: access_token_ttl.timestamp(),
        };

        let access_token = issue_access(&jwt_enc, claims)?;

        Ok(LoginResponse {
            access_token,
            access_token_expires_at: access_token_ttl,
            refresh_token: format!("{}.{}", new_session.id, refresh_token.raw),
            refresh_token_expires_at: expires_at,
        })
    }
}
