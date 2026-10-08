use std::net::{IpAddr, SocketAddr};

use crate::{error::ApiError, extractors::validate::ValidatedBodyJson, state::AppState};
use application::dto::auth_dto::{ClientContext, LoginRequest};
use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::IntoResponse,
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use chrono::{DateTime, Utc};

// Đọc IP client: ưu tiên `X-Forwarded-For` (token đầu) → `X-Real-IP` → địa chỉ peer.
// Chỉ chấp nhận giá trị parse được thành `IpAddr` để tránh fail cast `::inet` (500).
fn client_ip(headers: &HeaderMap, peer: SocketAddr) -> String {
    if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok())
        && let Some(first) = xff.split(',').next()
        && first.trim().parse::<IpAddr>().is_ok()
    {
        return first.trim().to_string();
    }
    if let Some(xrip) = headers.get("x-real-ip").and_then(|v| v.to_str().ok())
        && xrip.trim().parse::<IpAddr>().is_ok()
    {
        return xrip.trim().to_string();
    }
    peer.ip().to_string()
}

fn user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

pub struct CookieOptions<'a> {
    pub path: &'a str,
    pub same_site: SameSite,
    pub secure: bool,
    pub domain: Option<&'a str>,
    pub expires_at: Option<DateTime<Utc>>,
}

pub fn build_cookie(
    name: impl Into<String>,
    value: impl Into<String>,
    opts: CookieOptions<'_>,
) -> Cookie<'static> {
    let mut builder = Cookie::build((name.into(), value.into()))
        .http_only(true)
        .secure(opts.secure)
        .same_site(opts.same_site)
        .path(opts.path.to_owned());

    if let Some(d) = opts.domain.filter(|d| !d.trim().is_empty()) {
        builder = builder.domain(d.to_owned());
    }

    if let Some(expiry) = opts.expires_at {
        let remaining = (expiry - Utc::now()).num_seconds().max(0);
        builder = builder.max_age(time::Duration::seconds(remaining));

        if let Ok(t) = time::OffsetDateTime::from_unix_timestamp(expiry.timestamp()) {
            builder = builder.expires(t);
        }
    }

    builder.build()
}

pub async fn login_handler(
    jar: CookieJar,
    headers: HeaderMap,
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    ValidatedBodyJson(payload): ValidatedBodyJson<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let ctx = ClientContext {
        user_agent: user_agent(&headers),
        ip_address: Some(client_ip(&headers, peer)),
    };

    let response = state.auth_service.login(payload, ctx).await?;

    let access_cookie = build_cookie(
        state.config.access_token_name.clone(),
        response.access_token.clone(),
        CookieOptions {
            path: "/",
            same_site: SameSite::Lax,
            secure: state.config.cookie_secure,
            domain: state.config.cookie_domain.as_deref(),
            expires_at: Some(response.access_token_expires_at),
        },
    );

    let refresh_cookie = build_cookie(
        state.config.refresh_token_name.clone(),
        response.refresh_token.clone(),
        CookieOptions {
            path: "/api/v1/auth", // chỉ gửi cho các route auth
            same_site: SameSite::Strict,
            secure: state.config.cookie_secure,
            domain: state.config.cookie_domain.as_deref(),
            expires_at: Some(response.refresh_token_expires_at),
        },
    );

    let jar = jar.add(access_cookie).add(refresh_cookie);
    Ok((jar, Json(response)))
}
