use crate::{
    error::ApiError,
    extractor::validate::ValidatedBodyJson,
    helper::cookie::{auth_response, client_ip, user_agent},
    state::AppState,
};
use application::dto::auth::{ClientContext, LoginRequest};
use axum::{
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
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

    let response = state
        .auth_service
        .login(
            payload,
            ctx,
            state.config.access_token_ttl_secs,
            state.config.refresh_token_ttl_secs,
            state.config.max_refresh_token_ttl_secs,
            state.config.jwt_enc.clone(),
        )
        .await?;

    Ok(auth_response(jar, &state, device_type, response))
}
