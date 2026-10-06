use chrono::Duration;
use std::str::FromStr;

use crate::{
    dto::auth_dto::{ClientContext, LoginRequest, LoginResponse},
    errors::AppError,
    security::tokens::new_refresh,
};
use argon2::PasswordVerifier;
use domain::{
    entities::{
        session::{DeviceType, NewSession},
        user::UserStatus,
    },
    repositories::{SessionRepo, UserRepo},
};

pub struct AuthService<UR, USR>
where
    UR: UserRepo,
    USR: SessionRepo,
{
    user_repo: UR,
    session_repo: USR,
    session_ttl: Duration,
}

impl<UR, USR> AuthService<UR, USR>
where
    UR: UserRepo,
    USR: SessionRepo,
{
    pub fn new(user_repo: UR, session_repo: USR, session_ttl: Duration) -> Self {
        Self {
            user_repo,
            session_repo,
            session_ttl,
        }
    }

    pub async fn login(
        &self,
        request: LoginRequest,
        ctx: ClientContext,
    ) -> Result<LoginResponse, AppError> {
        let user = self
            .user_repo
            .find_by_email(&request.email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Email hoặc mật khẩu không đúng".into()))?;

        match user.status {
            UserStatus::Active => {}
            UserStatus::PendingPassword => {
                return Err(AppError::Unauthorized(
                    "Tài khoản chưa đặt mật khẩu. Vui lòng kiểm tra email.".into(),
                ));
            }
            UserStatus::Deactivated => {
                return Err(AppError::Unauthorized("Tài khoản đã bị vô hiệu hóa".into()));
            }
        }

        let hash_str = user.password_hash.clone().ok_or_else(|| {
            AppError::Unauthorized("Tài khoản chưa kích hoạt. Vui lòng kiểm tra email.".into())
        })?;

        let ok = tokio::task::spawn_blocking(move || {
            argon2::PasswordHash::new(&hash_str)
                .map(|h| {
                    argon2::Argon2::default()
                        .verify_password(request.password.as_bytes(), &h)
                        .is_ok()
                })
                .unwrap_or(false)
        })
        .await
        .map_err(|e| AppError::Internal(format!("Invalid password hash: {}", e)))?;
        if !ok {
            return Err(AppError::Unauthorized(
                "Email hoặc mật khẩu không đúng".into(),
            ));
        }

        let token = new_refresh();
        let expires_at = chrono::Utc::now() + self.session_ttl;

        let new_session = NewSession {
            user_id: user.id,
            refresh_token_hash: token.hash,
            device_type: DeviceType::from_str(&request.device_type)?,
            device_name: request.device_name,
            app_version: request.app_version,
            user_agent: ctx.user_agent,
            ip_address: ctx.ip_address,
            expires_at,
            absolute_expires_at: expires_at,
        };

        let new_session = self.session_repo.create(new_session).await?;

        // 5. Return response — trả token THÔ cho client
        Ok(LoginResponse {
            user_id: user.id.to_string(),
            session: format!("{}.{}", new_session.id, token.raw),
            expires_in: self.session_ttl.num_seconds(),
        })
    }
}
