// viết gọn
// pub trait UserRepo: Send + Sync {
//     fn find_by_id(
//         &self,
//         id: uuid::Uuid,
//     ) -> impl Future<Output = Result<Option<User>, RepositoryError>> + Send;
// }
// viết đầy đủ
// pub trait UserRepo
// where
//     Self: Send + Sync,
// {
//     fn find_by_id(
//         &self,
//         id: uuid::Uuid,
//     ) -> impl Future<Output = Result<Option<User>, RepositoryError>> + Send;
// }
// 2 cái trên là như nhau
use crate::{
    entities::{
        role::{Role, RolePermissions},
        session::{NewSession, Session},
        user::{CreateUserWithRoles, User},
    },
    errors::DomainError,
};
use chrono::{DateTime, Utc};
use uuid;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("không tìm thấy")]
    NotFound,

    #[error("vi phạm unique: {0}")]
    UniqueViolation(String), // mang tên constraint, để application tự dịch message

    #[error("vi phạm khóa ngoại: {0}")]
    ForeignKeyViolation(String),

    #[error("dữ liệu không hợp lệ: {0}")]
    Mapping(#[from] DomainError), // row -> entity fail (vd InvalidUserStatus)

    #[error(transparent)]
    Unexpected(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

pub trait UserRepo: Send + Sync {
    fn find_by_id(
        &self,
        id: uuid::Uuid,
    ) -> impl Future<Output = Result<Option<User>, RepositoryError>> + Send;

    fn find_by_email(
        &self,
        email: &str,
    ) -> impl Future<Output = Result<Option<User>, RepositoryError>> + Send;

    fn create_with_roles(
        &self,
        payload: CreateUserWithRoles,
    ) -> impl Future<Output = Result<User, RepositoryError>> + Send;
}

pub trait SessionRepo: Send + Sync {
    fn create(
        &self,
        new_session: NewSession,
    ) -> impl Future<Output = Result<Session, RepositoryError>> + Send;

    fn find_by_id(
        &self,
        id: uuid::Uuid,
    ) -> impl Future<Output = Result<Option<Session>, RepositoryError>> + Send;

    fn touch_last_seen(
        &self,
        id: uuid::Uuid,
        last_seen_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<bool, RepositoryError>> + Send;

    fn rotate(
        &self,
        id: uuid::Uuid,
        old_hash: &str,
        new_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<Option<Session>, RepositoryError>> + Send;

    fn revoke(
        &self,
        id: uuid::Uuid,
        reason: &str,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;
}

pub trait RoleRepo: Send + Sync {
    /// Mã permission của một user (JOIN user_roles→role_permissions→permissions),
    /// chỉ tính role đang ACTIVE và chưa xoá mềm.
    fn find_permission_codes_for_user(
        &self,
        user_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Vec<String>, RepositoryError>> + Send;

    /// Các role ACTIVE, chưa xoá mềm trong `role_ids`, kèm mã permission của từng role.
    /// Role không tồn tại / bị vô hiệu sẽ không có trong kết quả.
    fn find_active_role_permissions(
        &self,
        role_ids: &[uuid::Uuid],
    ) -> impl Future<Output = Result<Vec<RolePermissions>, RepositoryError>> + Send;

    fn find_by_id(
        &self,
        id: uuid::Uuid,
    ) -> impl Future<Output = Result<Option<Role>, RepositoryError>> + Send;

    fn find_permission_codes_for_role(
        &self,
        id: uuid::Uuid,
    ) -> impl Future<Output = Result<Vec<String>, RepositoryError>> + Send;

    /// Lọc ra những code thật sự có trong bảng `permissions`.
    fn find_existing_permission_codes(
        &self,
        codes: &[String],
    ) -> impl Future<Output = Result<Vec<String>, RepositoryError>> + Send;

    /// Thay toàn bộ permission của role trong 1 transaction.
    /// `NotFound` nếu role không tồn tại / không cho sửa / có code không tồn tại.
    fn replace_permissions(
        &self,
        role_id: uuid::Uuid,
        codes: &[String],
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;
}
