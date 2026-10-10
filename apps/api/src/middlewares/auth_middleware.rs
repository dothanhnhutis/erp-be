use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;

use crate::AppState;
use crate::error::ApiError;
use crate::extractors::jwt::Authentication;

/// Middleware bắt buộc đăng nhập: lấy token từ Bearer/cookie, xác thực, gắn AuthContext.
pub async fn require_auth(
    State(state): State<AppState>,
    Authentication(jwt): Authentication,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // let (session, user, permission_codes) = state.auth_service.authenticate(&token).await?;

    // req.extensions_mut().insert(AuthContext {
    //     session,
    //     user,
    //     permission_codes,
    // });

    Ok(next.run(req).await)
}
