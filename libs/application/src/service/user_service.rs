use domain::{entities::user::CreateUserWithRoles, repositories::UserRepo};

use crate::{
    dto::user_dto::{CreateUserRequest, CreateUserResponse},
    errors::AppError,
    security::authorization::Actor,
};

use domain::repositories::RoleRepo;

pub struct UserService<UR, RR>
where
    UR: UserRepo,
    RR: RoleRepo,
{
    user_repo: UR,
    role_repo: RR,
}

impl<UR, RR> UserService<UR, RR>
where
    UR: UserRepo,
    RR: RoleRepo,
{
    pub fn new(user_repo: UR, role_repo: RR) -> Self {
        Self {
            user_repo,
            role_repo,
        }
    }

    pub async fn create_user(
        &self,
        actor: &Actor,
        payload: CreateUserRequest,
    ) -> Result<CreateUserResponse, AppError> {
        let mut role_ids = payload.role_ids;
        role_ids.sort();
        role_ids.dedup();
        if role_ids.is_empty() {
            return Err(AppError::Validation("Phải chọn ít nhất một vai trò".into()));
        }

        // 1. Mọi role phải tồn tại và đang ACTIVE.
        let roles = self
            .role_repo
            .find_active_role_permissions(&role_ids)
            .await?;
        if roles.len() != role_ids.len() {
            return Err(AppError::Validation(
                "Vai trò không tồn tại hoặc đã bị vô hiệu".into(),
            ));
        }

        // 2. Không được gán role chứa quyền mà người tạo không có.
        actor.ensure_covers(roles.iter().flat_map(|r| r.codes.iter()), "gán vai trò")?;

        // 3. Tạo user + gán role (transaction trong repo).
        let user = self
            .user_repo
            .create_with_roles(CreateUserWithRoles {
                email: payload.email.trim().to_lowercase(),
                role_ids,
            })
            .await?;

        Ok(CreateUserResponse {
            user_id: user.id.to_string(),
        })
    }
}
