use chrono::Duration;
use jsonwebtoken::EncodingKey;
use shared::config::TokenConfig;
use std::str::FromStr;

use crate::{
    dto::auth_dto::{ClientContext, LoginRequest, LoginResponse},
    errors::AppError,
    security::tokens::{Claims, issue_access, new_refresh},
};
use argon2::PasswordVerifier;
use domain::{
    cache::SessionCache,
    entities::{
        cached_session::CachedSession,
        session::{DeviceType, NewSession, Session},
        user::{self, User, UserStatus},
    },
    repositories::{RoleRepository, SessionRepo, UserRepo},
};

pub struct AuthService<UR, USR, RR, SC>
where
    UR: UserRepo,
    USR: SessionRepo,
    RR: RoleRepository,
    SC: SessionCache,
{
    user_repo: UR,
    session_repo: USR,
    role_repo: RR,
    session_cache: SC,
    token: TokenConfig,
}

impl<UR, USR, RR, SC> AuthService<UR, USR, RR, SC>
where
    UR: UserRepo,
    USR: SessionRepo,
    RR: RoleRepository,
    SC: SessionCache,
{
    pub fn new(
        user_repo: UR,
        session_repo: USR,
        role_repo: RR,
        session_cache: SC,
        token: TokenConfig,
    ) -> Self {
        Self {
            user_repo,
            session_repo,
            role_repo,
            session_cache,
            token,
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
        let now = chrono::Utc::now();
        let access_token_ttl = now + self.token.access_ttl;
        let expires_at = now + self.token.refresh_ttl;
        let absolute_expires_at = now + self.token.max_refresh_ttl;

        let new_session = NewSession {
            user_id: user.id,
            refresh_token_hash: token.hash,
            device_type: DeviceType::from_str(&request.device_type)?,
            device_name: request.device_name,
            app_version: request.app_version,
            user_agent: ctx.user_agent,
            ip_address: ctx.ip_address,
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

        let access_token = issue_access(&self.token.jwt_enc, claims)?;

        Ok(LoginResponse {
            access_token,
            access_token_expires_at: access_token_ttl,
            refresh_token: format!("{}.{}", new_session.id, token.raw),
            refresh_token_expires_at: expires_at,
        })
    }

    pub async fn authentication(
        &self,
        claims: &Claims,
    ) -> Result<(Session, User, Vec<String>), AppError> {
        let is_revoked = self
            .session_cache
            .is_revoked(claims.sid)
            .await
            .map_err(|e| {
                tracing::warn!("cache sismember lỗi: {e}");
                AppError::Unauthorized("Phiên đăng nhập đã bị thu hồi".into())
            })?;

        if is_revoked {
            return Err(AppError::Unauthorized(
                "Phiên đăng nhập đã bị thu hồi".into(),
            ));
        }

        // 1. Cache trước; lỗi cache → coi như miss (fail-open), fallback DB.
        let cached = match self
            .session_cache
            .get(
                claims.sub.to_string().as_ref(),
                claims.sid.to_string().as_ref(),
            )
            .await
        {
            Ok(hit) => hit,
            Err(e) => {
                tracing::warn!("cache get lỗi, fallback DB: {e}");
                None
            }
        };

        let (session, user, db_synced_at) = match cached {
            Some(c) => (c.session, c.user, c.db_synced_at),
            None => {
                let (session, user) = tokio::try_join!(
                    async {
                        self.session_repo
                            .find_by_id(claims.sid)
                            .await?
                            .ok_or(AppError::Unauthorized("Thiếu thông tin xác thực".into()))
                    },
                    async {
                        self.user_repo
                            .find_by_id(claims.sub)
                            .await?
                            .ok_or(AppError::Unauthorized("Thiếu thông tin xác thực".into()))
                    }
                )?;
                (session, user, session.updated_at)
            }
        };

        // let (session, user, permission_codes) = tokio::try_join!(
        //     async {
        //         self.session_repo
        //             .find_by_id(claims.sid)
        //             .await?
        //             .ok_or(AppError::Unauthorized("Thiếu thông tin xác thực".into()))
        //     },
        //     async {
        //         self.user_repo
        //             .find_by_id(claims.sub)
        //             .await?
        //             .ok_or(AppError::Unauthorized("Thiếu thông tin xác thực".into()))
        //     },
        //     async {
        //         self.role_repo
        //             .find_permission_codes_for_user(claims.sub)
        //             .await
        //             .map_err(AppError::from)
        //     },
        // )?;

        let now = chrono::Utc::now();

        if session.revoked_at.is_some() {
            // let _ = self.cache.remove(&token_hash).await;
            return Err(AppError::Unauthorized(
                "Phiên đăng nhập đã bị thu hồi".into(),
            ));
        }
        if session.expires_at <= now {
            // let _ = self.cache.remove(&token_hash).await;
            // TODO:  vô hiệu hoá session db luôn  (revoke_reason 'EXPIRED')
            return Err(AppError::Unauthorized("Phiên đăng nhập đã hết hạn".into()));
        }
        if user.status == UserStatus::Deactivated {
            // let _ = self.cache.remove(&token_hash).await;
            // TODO: vô hiệu hoá session db luôn  (revoke_reason 'EXPIRED')
            return Err(AppError::Unauthorized("Tài khoản đã bị vô hiệu hóa".into()));
        }

        // let cached_session = CachedSession {
        //     session,
        //     user,
        //     db_synced_at: now,
        // };

        // self.session_repo.find_by_token_hash(token_hash)

        Ok((session, user, permission_codes))
    }
}
