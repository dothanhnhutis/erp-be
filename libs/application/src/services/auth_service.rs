use std::sync::LazyLock;

use crate::{
    dto::auth::{ClientContext, LoginRequest, LoginResponse},
    errors::AppError,
    security::{
        password::{hash_password, verify_password},
        tokens::{Claims, issue_access, new_refresh},
    },
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use domain::{
    cache::{PermissionCache, SessionCache, SessionLookup},
    entities::{
        cached_session::CachedSession,
        session::{NewSession, Session},
        user::{User, UserStatus},
    },
    repositories::{RoleRepo, SessionRepo, UserRepo},
};
use rand::Rng;
use shared::config::{SessionConfig, TokenConfig};

static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    hash_password(&URL_SAFE_NO_PAD.encode(bytes)).expect("hash dummy password")
});

pub struct AuthServiceInit<UR, SR, RR, SC, PC> {
    pub user_repo: UR,
    pub session_repo: SR,
    pub role_repo: RR,
    pub session_cache: SC,
    pub perm_cache: PC,
    pub token: TokenConfig,
    pub session: SessionConfig,
}

pub struct AuthService<UR, SR, RR, SC, PC>
where
    UR: UserRepo,
    SR: SessionRepo,
    RR: RoleRepo,
    SC: SessionCache,
    PC: PermissionCache,
{
    user_repo: UR,
    session_repo: SR,
    role_repo: RR,
    session_cache: SC,
    perm_cache: PC,
    token: TokenConfig,
    session: SessionConfig,
}

impl<UR, SR, RR, SC, PC> AuthService<UR, SR, RR, SC, PC>
where
    UR: UserRepo,
    SR: SessionRepo,
    RR: RoleRepo,
    SC: SessionCache,
    PC: PermissionCache,
{
    pub fn new(init: AuthServiceInit<UR, SR, RR, SC, PC>) -> Self {
        LazyLock::force(&DUMMY_HASH);
        Self {
            user_repo: init.user_repo,
            session_repo: init.session_repo,
            role_repo: init.role_repo,
            session_cache: init.session_cache,
            perm_cache: init.perm_cache,
            token: init.token,
            session: init.session,
        }
    }

    pub async fn login(
        &self,
        payload: LoginRequest,
        client_context: ClientContext,
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
        let access_token_ttl = now + self.token.access_ttl;
        let expires_at = now + self.token.refresh_ttl;
        let absolute_expires_at = now + self.token.max_refresh_ttl;
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

        let access_token = issue_access(&self.token.jwt_enc, claims)?;

        Ok(LoginResponse {
            access_token,
            access_token_expires_at: access_token_ttl,
            refresh_token: format!("{}.{}", new_session.id, refresh_token.raw),
            refresh_token_expires_at: expires_at,
        })
    }

    fn tombstone_ttl(&self) -> i64 {
        // access token cũ sống tối đa access_ttl (+ leeway 30s khi verify) → dư 60s
        self.token.access_ttl.num_seconds() + 60
    }

    async fn mark_revoked_cache(&self, user_id: uuid::Uuid, session_id: uuid::Uuid) {
        if let Err(e) = self
            .session_cache
            .mark_revoked(
                user_id.to_string().as_ref(),
                session_id.to_string().as_ref(),
                self.tombstone_ttl(),
            )
            .await
        {
            tracing::warn!("cache mark_revoked lỗi: {e}");
        }
    }

    async fn load_permissions(&self, user_id: uuid::Uuid) -> Result<Vec<String>, AppError> {
        // Lấy version TRƯỚC khi query DB.
        let version = match self.perm_cache.get(user_id).await {
            Ok((_, Some(codes))) => return Ok(codes),
            Ok((ver, None)) => Some(ver),
            Err(e) => {
                tracing::warn!("perm cache get lỗi, fallback DB: {e}");
                None // Redis lỗi → không ghi cache
            }
        };

        let codes = self
            .role_repo
            .find_permission_codes_for_user(user_id)
            .await?;

        if let Some(ver) = version
            && let Err(e) = self
                .perm_cache
                .put(
                    ver,
                    user_id,
                    &codes,
                    self.session.cache_ttl.num_seconds() as u64,
                )
                .await
        {
            tracing::warn!("perm cache put lỗi: {e}");
        }
        Ok(codes)
    }

    pub async fn authentication(
        &self,
        claims: &Claims,
    ) -> Result<(Session, User, Vec<String>), AppError> {
        let sub = claims.sub.to_string();
        let sid = claims.sid.to_string();

        let cached = match self.session_cache.get(&sub, &sid).await {
            Ok(SessionLookup::Hit(c)) => Some(*c),
            Ok(SessionLookup::Revoked) => {
                return Err(AppError::Unauthorized(
                    "Phiên đăng nhập đã bị thu hồi".into(),
                ));
            }
            Ok(SessionLookup::Miss) => None,
            Err(e) => {
                tracing::warn!("cache get lỗi, fallback DB: {e}");
                None
            }
        };

        let (mut session, user, cache_miss) = match cached {
            Some(c) => (c.session, c.user, false),
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
                (session, user, true)
            }
        };

        let now = chrono::Utc::now();

        if session.revoked_at.is_some() {
            self.mark_revoked_cache(claims.sub, claims.sid).await;
            return Err(AppError::Unauthorized(
                "Phiên đăng nhập đã bị thu hồi".into(),
            ));
        }

        let invalid = if session.expires_at <= now {
            Some(("EXPIRED", "Phiên đăng nhập đã hết hạn"))
        } else if user.status != UserStatus::Active {
            Some(("FORCED", "Tài khoản đã bị vô hiệu hóa"))
        } else {
            None
        };
        if let Some((reason, msg)) = invalid {
            if let Err(e) = self.session_repo.revoke(session.id, reason).await {
                tracing::warn!("revoke {reason} lỗi: {e}");
            }
            self.mark_revoked_cache(claims.sub, claims.sid).await;
            return Err(AppError::Unauthorized(msg.into()));
        }

        // 3. Throttle ghi DB; lỗi DB chỉ log.
        let mut touched = false;
        if now - session.last_seen_at >= self.session.db_sync {
            match self.session_repo.touch_last_seen(session.id, now).await {
                Ok(true) => {
                    session.last_seen_at = now;
                    touched = true;
                }
                Ok(false) => {
                    self.mark_revoked_cache(claims.sub, claims.sid).await;
                    return Err(AppError::Unauthorized(
                        "Phiên đăng nhập đã bị thu hồi".into(),
                    ));
                }
                Err(e) => tracing::warn!("touch_last_seen lỗi: {e}"),
            }
        }

        // 4. Refresh cache.
        if cache_miss || touched {
            let entry = CachedSession {
                session: session.clone(),
                user: user.clone(),
            };
            match self
                .session_cache
                .put(&sub, &sid, &entry, self.session.cache_ttl.num_seconds())
                .await
            {
                Ok(true) => {}
                Ok(false) => {
                    return Err(AppError::Unauthorized(
                        "Phiên đăng nhập đã bị thu hồi".into(),
                    ));
                }
                Err(e) => tracing::warn!("cache put lỗi: {e}"),
            }
        }

        let permission_codes = self.load_permissions(claims.sub).await?;

        Ok((session, user, permission_codes))
    }

    pub async fn logout(&self, session: &Session) -> Result<(), AppError> {
        self.session_repo.revoke(session.id, "LOGOUT").await?;
        self.mark_revoked_cache(session.user_id, session.id).await;
        Ok(())
    }
}
