use domain::{
    cache::PermissionCache,
    repositories::{RepositoryError, RoleRepo},
};

use crate::{
    dto::role_dto::UpdateRolePermissionsRequest, errors::AppError, security::authorization::Actor,
};

pub struct RoleService<RR, PC>
where
    RR: RoleRepo,
    PC: PermissionCache,
{
    role_repo: RR,
    perm_cache: PC,
}

impl<RR, PC> RoleService<RR, PC>
where
    RR: RoleRepo,
    PC: PermissionCache,
{
    pub fn new(role_repo: RR, perm_cache: PC) -> Self {
        Self {
            role_repo,
            perm_cache,
        }
    }

    pub async fn update_permissions(
        &self,
        actor: &Actor,
        role_id: uuid::Uuid,
        payload: UpdateRolePermissionsRequest,
    ) -> Result<(), AppError> {
        let mut new_codes: Vec<String> = payload
            .permission_codes
            .into_iter()
            .map(|c| c.trim().to_owned())
            .collect();
        new_codes.sort();
        new_codes.dedup();

        let role = self
            .role_repo
            .find_by_id(role_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Vai trò không tồn tại".into()))?;

        // Lớp 1
        if !role.can_update {
            return Err(AppError::Forbidden("Không thể sửa vai trò hệ thống".into()));
        }

        // Lớp 2a + 2b — kiểm tra quyền TRƯỚC khi báo code nào không tồn tại.
        let current = self
            .role_repo
            .find_permission_codes_for_role(role_id)
            .await?;
        // Mọi code phải tồn tại.
        let existing = self
            .role_repo
            .find_existing_permission_codes(&new_codes)
            .await?;
        actor.ensure_covers(&current, "sửa vai trò này")?;
        actor.ensure_covers(&new_codes, "cấp quyền")?;

        if existing.len() != new_codes.len() {
            let unknown: Vec<&str> = new_codes
                .iter()
                .filter(|c| !existing.contains(c))
                .map(String::as_str)
                .collect();
            return Err(AppError::Validation(format!(
                "Quyền không tồn tại: {}",
                unknown.join(", ")
            )));
        }

        self.role_repo
            .replace_permissions(role_id, &new_codes)
            .await
            .map_err(|e| match e {
                // Race: role vừa bị xoá / bị khoá sửa giữa lúc kiểm tra và ghi.
                RepositoryError::NotFound => AppError::NotFound("Vai trò không tồn tại".into()),
                e => e.into(),
            })?;

        // Quyền của mọi user giữ role này đổi → bỏ cache cũ. DB đã đúng nên chỉ log.
        if let Err(e) = self.perm_cache.bump_version().await {
            tracing::warn!("bump perm version lỗi: {e}");
        }
        Ok(())
    }
}
