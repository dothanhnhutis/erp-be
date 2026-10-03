use std::net::{IpAddr, SocketAddr};

use crate::{extractors::validate::ValidatedBodyJson, state::AppState};
use application::dto::auth_dto::ClientContext;
use axum::{
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::IntoResponse,
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use domain::repositories::UserRepo;
use serde::Deserialize;
use validator::{Validate, ValidationError};

#[derive(Debug, Validate, Deserialize)]
pub struct LoginPayload {
    #[validate(email)]
    email: String,
    #[validate(
        length(min = 1, message = "Email và mật khẩu không hợp lệ."),
        custom(function = "validate_password")
    )]
    pub password: String,
    pub app_version: Option<String>,
    pub platform: Option<String>,
    #[validate(custom(function = "validate_device_type"))]
    pub device_type: String,
    pub device_name: Option<String>,
    pub device_id: Option<String>,
}

fn validate_password(password: &str) -> Result<(), ValidationError> {
    let mut has_uppercase = false;
    let mut has_lowercase = false;
    let mut has_digit = false;

    for c in password.chars() {
        has_uppercase |= c.is_ascii_uppercase();
        has_lowercase |= c.is_ascii_lowercase();
        has_digit |= c.is_ascii_digit();
    }
    if !password.is_empty() && has_uppercase && has_lowercase && has_digit {
        Ok(())
    } else {
        Err(ValidationError {
            code: "invalid_password".into(),
            message: Some("Email và mật khẩu không hợp lệ.".into()),
            params: Default::default(),
        })
    }
}

fn validate_device_type(device_type: &str) -> Result<(), ValidationError> {
    let device_types = ["web", "mobile", "desktop"];

    if device_types.contains(&device_type) {
        Ok(())
    } else {
        let mut error = ValidationError::new("invalid_device_type");
        error.message = Some("Loại thiết bị không hợp lệ.".into());
        Err(error)
    }
}

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

// Cookie `session` với thuộc tính dùng chung. `max_age = None` (kèm value rỗng) dùng để xóa cookie —
// PHẢI khớp `path`/`domain` với cookie lúc login thì trình duyệt mới gỡ.
fn session_cookie(
    value: String,
    secure: bool,
    domain: Option<&str>,
    max_age: Option<time::Duration>,
) -> Cookie<'static> {
    let mut builder = Cookie::build(("session", value))
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .path("/");

    if let Some(d) = domain {
        builder = builder.domain(d.to_owned());
    }
    if let Some(age) = max_age {
        builder = builder.max_age(age);
    }

    builder.build()
}

pub async fn login_handler(
    jar: CookieJar,
    headers: HeaderMap,
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    ValidatedBodyJson(payload): ValidatedBodyJson<LoginPayload>,
) -> impl IntoResponse {
    let ctx = ClientContext {
        user_agent: user_agent(&headers),
        ip_address: Some(client_ip(&headers, peer)),
    };

    // let user = state
    //     .pg_user_repo
    //     .find_by_email("dothanhnhutis@gmail.com")
    //     .await;

    let response = state.auth_service.login(payload, ctx).await?;

    println!("{:#?}", user);
    println!("{:#?}", payload);

    // (CookieJar, Json): jar là IntoResponseParts nên đứng trước body.
    Ok((jar.add(cookie), Json(response)))
}
