use application::{dto::auth_dto::ClientContext, errors::AppError};
use axum::{
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use std::net::SocketAddr;

use crate::{
    error::ApiError,
    extractors::refresh_token::RefreshToken,
    helpers::cookie::{auth_response, clear_auth_cookies, client_ip, user_agent},
    state::AppState,
};

pub async fn refresh_token_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    // RefreshToken { session_id, secret }: RefreshToken,
    token: Result<RefreshToken, ApiError>,
) -> Response {
    let result = async {
        let RefreshToken { session_id, secret } = token?;
        let ctx = ClientContext {
            user_agent: user_agent(&headers),
            ip_address: Some(client_ip(&headers, peer)),
        };
        Ok::<_, ApiError>(state.auth_service.refresh(session_id, &secret, ctx).await?)
    }
    .await;

    match result {
        Ok((response, device_type)) => auth_response(jar, &state, device_type, response),
        Err(e @ ApiError::Domain(AppError::Unauthorized(_))) => {
            (clear_auth_cookies(jar, &state), e).into_response()
        }
        Err(e) => e.into_response(),
    }
}
