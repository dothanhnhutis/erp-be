use crate::{
    entities::{
        session::{NewSession, Session},
        user::User,
    },
    errors::DomainError,
};

#[derive(thiserror::Error, Debug)]
pub enum RepoError {
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

pub trait UserRepo {
    fn find_by_email(
        &self,
        email: &str,
    ) -> impl Future<Output = Result<Option<User>, RepoError>> + Send;
}

pub trait SessionRepo {
    fn create(
        &self,
        payload: NewSession,
    ) -> impl Future<Output = Result<Session, RepoError>> + Send;
}
