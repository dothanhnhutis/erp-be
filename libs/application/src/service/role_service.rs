use domain::repositories::RoleRepo;

use crate::{errors::AppError, security::authorization::Actor};

pub struct RoleService<RR>
where
    RR: RoleRepo,
{
    role_repo: RR,
}

impl<RR> RoleService<RR>
where
    RR: RoleRepo,
{
    pub fn new(role_repo: RR) -> Self {
        Self { role_repo }
    }

    // pub async fn update_permissions(
    //     &self,
    //     actor: &Actor,
    //     role_id: uuid::Uuid,
    //     new_codes: Vec<String>,
    // ) -> Result<(), AppError> {
    //     let role = self
    //         .role_repo
    //         .find_by_id(role_id)
    //         .await?
    //         .filter(|r| r.deleted_at.is_none())
    //         .ok_or_else(|| AppError::NotFound("Vai trò không tồn tại".into()))?;

    //     // Lớp 1: role hệ thống — không ai được sửa, kể cả admin.
    //     if !role.can_update {
    //         return Err(AppError::Forbidden("Không thể sửa vai trò hệ thống".into()));
    //     }

    //     // Lớp 2a: role hiện tại không được mạnh hơn actor (không đụng vào role "cấp trên").
    //     let current = self
    //         .role_repo
    //         .find_permission_codes_for_role(role_id)
    //         .await?;
    //     actor.ensure_covers(&current, "sửa vai trò này")?;

    //     // Lớp 2b: không được cấp quyền mình không có.
    //     actor.ensure_covers(&new_codes, "cấp quyền")?;

    //     // ... kiểm tra mọi code trong new_codes tồn tại, rồi thay role_permissions trong 1 transaction ...

    //     // Quyền của mọi user giữ role này đổi → làm mới cache permission.
    //     // self.perm_cache.bump_version().await  (log warn nếu lỗi)
    //     Ok(())
    // }
}
