use application::dto::user_dto::CreateUserRequest;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{
    error::ApiError,
    extractors::{actor::CurrentActor, required::Require, validate::ValidatedBodyJson},
    permissions::UserCreate,
    state::AppState,
};

pub async fn create_usere_handler(
    State(state): State<AppState>,
    _: Require<UserCreate>,
    CurrentActor(actor): CurrentActor,
    ValidatedBodyJson(payload): ValidatedBodyJson<CreateUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.user_service.create_user(&actor, payload).await?;
    Ok((StatusCode::CREATED, Json(res)))
}
