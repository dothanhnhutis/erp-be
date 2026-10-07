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
    ValidatedBodyJson(payload): ValidatedBodyJson<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let ctx = ClientContext {
        user_agent: user_agent(&headers),
        ip_address: Some(client_ip(&headers, peer)),
    };

    let response = state.auth_service.login(payload, ctx).await?;

    println!("{:#?}", response);
    // println!("{:#?}", payload);

    let cookie = session_cookie(
        response.session.clone(),
        false,
        Some(""),
        Some(time::Duration::seconds(response.expires_in)),
    );

    // (CookieJar, Json): jar là IntoResponseParts nên đứng trước body.
    Ok((jar.add(cookie), Json(response)))
}
