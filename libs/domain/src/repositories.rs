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
        session::{NewSession, Session},
        user::User,
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

    fn touch_expires(
        &self,
        id: uuid::Uuid,
        expires_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;

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

pub trait RoleRepository: Send + Sync {
    /// Mã permission của một user (JOIN user_roles→role_permissions→permissions),
    /// chỉ tính role đang ACTIVE và chưa xoá mềm.
    fn find_permission_codes_for_user(
        &self,
        user_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Vec<String>, RepositoryError>> + Send;
}
