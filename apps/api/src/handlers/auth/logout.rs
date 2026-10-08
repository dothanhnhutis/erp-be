use axum::extract::State;
use axum_extra::extract::CookieJar;

use crate::state::AppState;

pub async fn logout_handler(
    State(state): State<AppState>,
    data: AuthContext,
    jar: CookieJar,
) -> Result<impl IntoResponse, ApiError> {
    state
        .auth_service
        .logout(data.session.id, &data.session.token_hash)
        .await?;

    let removal = session_cookie(
        String::new(),
        state.config.cookie_secure,
        state.config.cookie_domain.as_deref(),
        None,
    );
    Ok((
        jar.remove(removal),
        Json(json!({ "message": "Đã đăng xuất" })),
    ))
}
