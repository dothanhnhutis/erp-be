use domain::cache::CacheError;
use domain::repositories::RepoError;
use redis::ServerErrorKind;
use redis::{self};
use sqlx::error::ErrorKind;

pub fn map_sqlx_error(err: sqlx::Error) -> RepoError {
    if let Some(db) = err.as_database_error() {
        match db.kind() {
            ErrorKind::UniqueViolation => {
                return RepoError::UniqueViolation(db.constraint().unwrap_or_default().to_owned());
            }
            ErrorKind::ForeignKeyViolation => {
                return RepoError::ForeignKeyViolation(
                    db.constraint().unwrap_or_default().to_owned(),
                );
            }
            _ => {}
        }
    }
    RepoError::Unexpected(Box::new(err))
}

pub fn map_redis(err: redis::RedisError) -> CacheError {
    // Nhóm "cache tạm không khả dụng" → degrade, fallback nguồn sự thật (DB)
    let transient = err.is_timeout()
        || err.is_connection_dropped()
        || err.is_connection_refusal()
        || err.is_io_error()
        || err.is_cluster_error()            // Moved / Ask / TryAgain / ClusterDown
        || matches!(
            err.kind(),
            redis::ErrorKind::Server(
                ServerErrorKind::BusyLoading   // server đang load dump
                    | ServerErrorKind::MasterDown
                    | ServerErrorKind::ReadOnly // ghi nhầm vào replica
            )
        );

    if transient {
        CacheError::Unavailable(Box::new(err))
    } else {
        CacheError::Unexpected(Box::new(err)) // ResponseError, ExecAbort, AuthFailed, config…
    }
}

pub fn map_json(err: serde_json::Error) -> CacheError {
    CacheError::Serialization(err.to_string())
}
