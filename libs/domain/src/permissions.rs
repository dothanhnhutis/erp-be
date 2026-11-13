//! Danh mục permission và role hệ thống — nguồn sự thật duy nhất.
//!
//! - Code phân quyền dùng các hằng trong [`codes`] (không gõ chuỗi tay).
//! - `erp-seeder system` đồng bộ [`ALL`] và [`SYSTEM_ROLES`] xuống DB (idempotent).
//! - Thêm permission mới: thêm hằng vào [`codes`] + một dòng vào [`ALL`], rồi chạy `erp-seeder system`.

/// Mã permission. Quy ước: `<ĐỐI_TƯỢNG>_<HÀNH_ĐỘNG>`, chữ hoa, gạch dưới.
pub mod codes {
    pub const USER_VIEW: &str = "USER_VIEW";
    pub const USER_CREATE: &str = "USER_CREATE";
    pub const USER_UPDATE: &str = "USER_UPDATE";
    pub const USER_DEACTIVATE: &str = "USER_DEACTIVATE";
    pub const USER_SESSION_REVOKE: &str = "USER_SESSION_REVOKE";

    pub const ROLE_VIEW: &str = "ROLE_VIEW";
    pub const ROLE_CREATE: &str = "ROLE_CREATE";
    pub const ROLE_UPDATE: &str = "ROLE_UPDATE";
    pub const ROLE_DELETE: &str = "ROLE_DELETE";
}

#[derive(Debug, Clone, Copy)]
pub struct PermissionDef {
    pub code: &'static str,
    pub description: &'static str,
}

pub const ALL: &[PermissionDef] = &[
    PermissionDef { code: codes::USER_VIEW, description: "Xem danh sách và chi tiết người dùng" },
    PermissionDef { code: codes::USER_CREATE, description: "Tạo người dùng" },
    PermissionDef { code: codes::USER_UPDATE, description: "Cập nhật thông tin và vai trò người dùng" },
    PermissionDef { code: codes::USER_DEACTIVATE, description: "Vô hiệu hoá / kích hoạt người dùng" },
    PermissionDef { code: codes::USER_SESSION_REVOKE, description: "Thu hồi phiên đăng nhập của người dùng khác" },
    PermissionDef { code: codes::ROLE_VIEW, description: "Xem vai trò" },
    PermissionDef { code: codes::ROLE_CREATE, description: "Tạo vai trò" },
    PermissionDef { code: codes::ROLE_UPDATE, description: "Cập nhật vai trò và quyền của vai trò" },
    PermissionDef { code: codes::ROLE_DELETE, description: "Xoá vai trò" },
];

/// Role hệ thống: định danh bằng `roles.code` (NOT NULL), id do DB sinh.
/// Luật: không xoá / vô hiệu hoá / đổi code; được đổi `name`, `description`
/// (DB chặn bằng trigger `trg_roles_protect_system`).
#[derive(Debug, Clone, Copy)]
pub struct SystemRoleDef {
    pub code: &'static str,
    /// Tên mặc định khi tạo lần đầu; sau đó admin đổi tên thì seeder không ghi đè.
    pub default_name: &'static str,
    pub description: &'static str,
    /// `true` → luôn được gán toàn bộ permission mỗi lần `system` chạy.
    pub grant_all_permissions: bool,
}

pub const SUPER_ADMIN: SystemRoleDef = SystemRoleDef {
    code: "SUPER_ADMIN",
    default_name: "Super Admin",
    description: "Quản trị toàn hệ thống, luôn có mọi quyền",
    grant_all_permissions: true,
};

pub const SYSTEM_ROLES: &[SystemRoleDef] = &[SUPER_ADMIN];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn permission_codes_are_unique_and_well_formed() {
        let mut seen = HashSet::new();
        for p in ALL {
            assert!(seen.insert(p.code), "trùng permission code: {}", p.code);
            assert!(
                p.code.len() <= 100
                    && p.code.chars().all(|c| c.is_ascii_uppercase() || c == '_'),
                "permission code sai quy ước: {}",
                p.code
            );
        }
    }

    #[test]
    fn system_role_codes_are_unique_and_fit_column() {
        let mut seen = HashSet::new();
        for r in SYSTEM_ROLES {
            assert!(seen.insert(r.code), "trùng role code: {}", r.code);
            assert!(r.code.len() <= 50, "role code quá dài: {}", r.code);
        }
    }
}
