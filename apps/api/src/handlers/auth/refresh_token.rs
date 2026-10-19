use application::dto::auth_dto::ClientContext;
use axum::{
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use std::net::{IpAddr, SocketAddr};

use crate::{error::ApiError, extractors::refresh_token::RefreshToken, state::AppState};

pub async fn refresh_token_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    RefreshToken { session_id, secret }: RefreshToken,
) -> Result<impl IntoResponse, ApiError> {
    // let ctx = ClientContext {
    //     user_agent: user_agent(&headers),
    //     ip_address: Some(client_ip(&headers, peer)),
    // };
    // let response = state.auth_service.refresh(session_id, &secret, ctx).await?;
    // set lại 2 cookie giống login, tái sử dụng build_cookie(...)
    Ok("ok")
}
