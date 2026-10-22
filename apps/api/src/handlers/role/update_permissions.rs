use application::dto::role_dto::UpdateRolePermissionsRequest;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    error::ApiError,
    extractors::{actor::CurrentActor, required::Require, validate::ValidatedBodyJson},
    permissions::RoleUpdate,
    state::AppState,
};

pub async fn update_role_permissions_handler(
    State(state): State<AppState>,
    _: Require<RoleUpdate>,
    CurrentActor(actor): CurrentActor,
    Path(role_id): Path<Uuid>,
    ValidatedBodyJson(payload): ValidatedBodyJson<UpdateRolePermissionsRequest>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .role_service
        .update_permissions(&actor, role_id, payload)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
