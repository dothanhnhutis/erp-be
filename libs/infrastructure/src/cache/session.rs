use domain::{
    cache::{CacheError, SessionCache},
    entities::cached_session::CachedSession,
};
use redis::{AsyncCommands, aio::ConnectionManager};

use crate::{
    cache::connect::session_key,
    persistence::error::{map_json, map_redis},
};
use domain::cache::SessionLookup;
use std::sync::LazyLock;

const TOMBSTONE: &str = "__revoked__";

/// Ghi entry trừ khi key đang là tombstone. Chạy nguyên tử trong Redis.
/// KEYS[1] = session key; ARGV[1] = json; ARGV[2] = ttl; ARGV[3] = tombstone
static SAFE_PUT: LazyLock<redis::Script> = LazyLock::new(|| {
    redis::Script::new(
        r#"
        if redis.call('GET', KEYS[1]) == ARGV[3] then return 0 end
        redis.call('SET', KEYS[1], ARGV[1], 'EX', ARGV[2])
        return 1
        "#,
    )
});

#[derive(Clone)]
pub struct RedisSessionCache {
    conn: ConnectionManager,
}

impl RedisSessionCache {
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }
}

impl SessionCache for RedisSessionCache {
    async fn get(&self, user_id: &str, session_id: &str) -> Result<SessionLookup, CacheError> {
        let mut conn = self.conn.clone();
        let raw: Option<String> = conn
            .get(session_key(user_id, session_id))
            .await
            .map_err(map_redis)?;
        Ok(match raw {
            None => SessionLookup::Miss,
            Some(s) if s == TOMBSTONE => SessionLookup::Revoked,
            Some(s) => SessionLookup::Hit(Box::new(serde_json::from_str(&s).map_err(map_json)?)),
        })
    }

    async fn put(
        &self,
        user_id: &str,
        session_id: &str,
        entry: &CachedSession,
        ttl_secs: i64,
    ) -> Result<bool, CacheError> {
        let mut conn = self.conn.clone();
        let json = serde_json::to_string(entry).map_err(map_json)?;
        let written: i32 = SAFE_PUT
            .key(session_key(user_id, session_id))
            .arg(json)
            .arg(ttl_secs.max(1))
            .arg(TOMBSTONE)
            .invoke_async(&mut conn)
            .await
            .map_err(map_redis)?;
        Ok(written == 1)
    }

    async fn mark_revoked(
        &self,
        user_id: &str,
        session_id: &str,
        ttl_secs: i64,
    ) -> Result<(), CacheError> {
        let mut conn = self.conn.clone();
        // Ghi đè VÔ ĐIỀU KIỆN: thay bản JSON đang có (nếu có) bằng tombstone.
        let _: () = conn
            .set_ex(
                session_key(user_id, session_id),
                TOMBSTONE,
                ttl_secs.max(1) as u64,
            )
            .await
            .map_err(map_redis)?;
        Ok(())
    }

    async fn mark_revoked_many(
        &self,
        user_id: &str,
        session_ids: &[uuid::Uuid],
        ttl_secs: i64,
    ) -> Result<(), CacheError> {
        if session_ids.is_empty() {
            return Ok(());
        }
        let mut conn = self.conn.clone();
        let mut pipe = redis::pipe();
        for sid in session_ids {
            pipe.set_ex(
                session_key(user_id, &sid.to_string()),
                TOMBSTONE,
                ttl_secs.max(1) as u64,
            )
            .ignore();
        }
        pipe.query_async::<()>(&mut conn).await.map_err(map_redis)?;
        Ok(())
    }
}
