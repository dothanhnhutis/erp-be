use crate::{
    error::ApiError,
    extractors::validate::ValidatedBodyJson,
    helpers::cookie::{CookieOptions, build_cookie, client_ip, user_agent},
    state::AppState,
};
use application::dto::auth_dto::{ClientContext, LoginRequest, WebLoginResponse};
use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::IntoResponse,
};
use axum_extra::extract::{CookieJar, cookie::SameSite};

use domain::entities::session::DeviceType;
use std::net::SocketAddr;
use std::str::FromStr;

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

    let device_type = DeviceType::from_str(&payload.device_type)?;

    let response = state.auth_service.login(payload, ctx).await?;

    match device_type {
        DeviceType::Web => {
            let access_cookie = build_cookie(
                state.config.access_token_name.clone(),
                response.access_token,
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
                response.refresh_token,
                CookieOptions {
                    path: "/api/v1/auth", // chỉ gửi cho các route auth
                    same_site: SameSite::Strict,
                    secure: state.config.cookie_secure,
                    domain: state.config.cookie_domain.as_deref(),
                    expires_at: Some(response.refresh_token_expires_at),
                },
            );

            // Web: token nằm trong httpOnly cookie, body KHÔNG trả token
            let body = WebLoginResponse {
                access_token_expires_at: response.access_token_expires_at,
                refresh_token_expires_at: response.refresh_token_expires_at,
            };

            let jar = jar.add(access_cookie).add(refresh_cookie);
            Ok((jar, Json(body)).into_response())
        }
        DeviceType::Mobile | DeviceType::Desktop => {
            // App: không có cookie, trả token trong body
            Ok(Json(response).into_response())
        }
    }
}
