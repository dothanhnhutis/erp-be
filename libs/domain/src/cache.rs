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
        user_id: &str,
        session_id: &str,
    ) -> impl Future<Output = Result<Option<CachedSession>, CacheError>> + Send;

    fn put(
        &self,
        user_id: &str,
        session_id: &str,
        entry: &CachedSession,
        ttl_secs: i64,
    ) -> impl Future<Output = Result<(), CacheError>> + Send;

    fn remove(
        &self,
        user_id: &str,
        session_id: &str,
    ) -> impl Future<Output = Result<(), CacheError>> + Send;

    // fn remove_all_for_user(
    //     &self,
    //     user_id: Uuid,
    // ) -> impl Future<Output = Result<(), CacheError>> + Send;

    // fn is_revoked(&self, id: uuid::Uuid) -> impl Future<Output = Result<bool, CacheError>> + Send;
}

/// Port cache permission theo user, có version toàn cục để invalidate hàng loạt.
pub trait PermissionCache: Send + Sync {
    /// Trả về (version hiện tại, codes nếu hit).
    /// Caller phải giữ version này để `put`, không đọc lại version sau khi query DB.
    fn get(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(u64, Option<Vec<String>>), CacheError>> + Send;

    fn put(
        &self,
        version: u64,
        user_id: Uuid,
        codes: &[String],
        ttl_secs: u64,
    ) -> impl Future<Output = Result<(), CacheError>> + Send;

    /// Gán/gỡ role cho 1 user → chỉ xóa key của user đó.
    fn invalidate_user(&self, user_id: Uuid)
    -> impl Future<Output = Result<(), CacheError>> + Send;

    /// Sửa role/permission → mọi key cũ bị bỏ, tự hết hạn theo TTL.
    fn bump_version(&self) -> impl Future<Output = Result<(), CacheError>> + Send;
}
