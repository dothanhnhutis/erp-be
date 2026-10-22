use std::collections::HashSet;
use uuid::Uuid;

use crate::errors::AppError;

/// Người đang thực hiện thao tác (lấy từ request đã xác thực).
pub struct Actor {
    pub user_id: Uuid,
    pub permissions: HashSet<String>,
}

impl Actor {
    /// Lớp 2: actor phải có MỌI quyền trong `codes` — không được thao tác lên thứ mạnh hơn mình.
    pub fn ensure_covers<'a>(
        &self,
        codes: impl IntoIterator<Item = &'a String>,
        action: &str,
    ) -> Result<(), AppError> {
        let mut missing: Vec<&str> = codes
            .into_iter()
            .map(String::as_str)
            .filter(|c| !self.permissions.contains(*c))
            .collect();
        if missing.is_empty() {
            return Ok(());
        }
        missing.sort_unstable();
        missing.dedup();
        Err(AppError::Forbidden(format!(
            "Không thể {action} vì liên quan tới quyền bạn không có: {}",
            missing.join(", ")
        )))
    }

    /// Không cho thao tác nguy hiểm lên chính mình (tự khoá, tự gỡ role).
    pub fn ensure_not_self(&self, target: Uuid, action: &str) -> Result<(), AppError> {
        if self.user_id == target {
            return Err(AppError::Forbidden(format!(
                "Không thể tự {action} chính mình"
            )));
        }
        Ok(())
    }
}
