use uuid::Uuid;

use crate::entities::cached_session::CachedSession;

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    /// (de)serialize value hỏng — bug hoặc lệch schema
    #[error("lỗi serialize cache: {0}")]
    Serialization(String),

    /// cache tạm không dùng được: timeout, mất/từ chối kết nối, đang load, cluster down
    #[error("cache tạm không khả dụng: {0}")]
    Unavailable(Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error(transparent)]
    Unexpected(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

/// Port cache phiên (outbound). Adapter (vd Redis) hiện thực ở tầng infrastructure.
pub trait SessionCache: Send + Sync {
    fn get(
        &self,
        token_hash: &str,
    ) -> impl Future<Output = Result<Option<CachedSession>, CacheError>> + Send;

    fn put(
        &self,
        token_hash: &str,
        entry: &CachedSession,
        ttl_secs: i64,
    ) -> impl Future<Output = Result<(), CacheError>> + Send;

    fn remove(&self, token_hash: &str) -> impl Future<Output = Result<(), CacheError>> + Send;

    fn remove_all_for_user(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), CacheError>> + Send;
}
