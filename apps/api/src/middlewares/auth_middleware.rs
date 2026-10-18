use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;

use crate::AppState;
use crate::error::ApiError;
use crate::extractors::jwt::Authentication;
use crate::extractors::permission::Permissions;
use crate::extractors::session::UserSession;
use crate::extractors::user::Me;

/// Middleware bắt buộc đăng nhập: lấy token từ Bearer/cookie, xác thực, gắn AuthContext.
pub async fn require_auth(
    State(state): State<AppState>,
    Authentication(jwt): Authentication,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let (session, user, codes) = state.auth_service.authentication(&jwt).await?;

    let ext = req.extensions_mut();
    ext.insert(Me(user));
    ext.insert(UserSession(session));
    ext.insert(Permissions(codes));

    Ok(next.run(req).await)
}
