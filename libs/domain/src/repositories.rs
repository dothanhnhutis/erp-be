//
// pub trait UserRepo: Send + Sync {
//     fn find_by_id(
//         &self,
//         id: uuid::Uuid,
//     ) -> impl Future<Output = Result<Option<User>, RepositoryError>> + Send;
// }

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
use crate::{entities::user::User, errors::DomainError};
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
