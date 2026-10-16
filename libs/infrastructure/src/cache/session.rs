use domain::{
    cache::{CacheError, SessionCache},
    entities::cached_session::CachedSession,
};
use redis::{AsyncCommands, aio::ConnectionManager};

use crate::{
    cache::connect::{session_key, session_revoked_key, user_key},
    persistence::error::{map_json, map_redis},
};

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
    async fn get(
        &self,
        user_id: &str,
        session_id: &str,
    ) -> Result<Option<CachedSession>, CacheError> {
        let mut conn: ConnectionManager = self.conn.clone();
        let raw: Option<String> = conn
            .get(session_key(user_id, session_id))
            .await
            .map_err(map_redis)?;
        match raw {
            Some(s) => Ok(Some(serde_json::from_str(&s).map_err(map_json)?)),
            None => Ok(None),
        }
    }

    // async fn put(
    //     &self,
    //     token_hash: &str,
    //     entry: &CachedSession,
    //     ttl_secs: i64,
    // ) -> Result<(), CacheError> {
    //     let mut conn = self.conn.clone();
    //     let json = serde_json::to_string(entry).map_err(map_json)?;
    //     let skey = session_key(token_hash);
    //     let ukey = user_key(entry.session.user_id);
    //     let ttl = ttl_secs.max(1); // SET EX 0 / EXPIRE 0 là footgun

    //     redis::pipe()
    //         .atomic()
    //         .set_ex(&skey, json, ttl as u64)
    //         .ignore()
    //         .sadd(&ukey, token_hash)
    //         .ignore()
    //         .expire(&ukey, ttl)
    //         .ignore()
    //         .query_async::<()>(&mut conn)
    //         .await
    //         .map_err(map_redis)?;
    //     Ok(())
    // }

    // async fn remove(&self, token_hash: &str) -> Result<(), CacheError> {
    //     let mut conn = self.conn.clone();
    //     // Chỉ xóa session key; phần tử thừa trong user set tự hết hạn theo TTL.
    //     let _: () = conn.del(session_key(token_hash)).await.map_err(map_redis)?;
    //     Ok(())
    // }

    // async fn remove_all_for_user(&self, x: Uuid) -> Result<(), CacheError> {
    //     let mut conn = self.conn.clone();
    //     let ukey = user_key(user_id);
    //     let hashes: Vec<String> = conn.smembers(&ukey).await.map_err(map_redis)?;

    //     let mut pipe = redis::pipe();
    //     for h in &hashes {
    //         pipe.del(session_key(h)).ignore();
    //     }
    //     pipe.del(&ukey).ignore();
    //     pipe.query_async::<()>(&mut conn).await.map_err(map_redis)?;
    //     Ok(())
    // }

    async fn is_revoked(&self, id: uuid::Uuid) -> Result<bool, CacheError> {
        let mut conn = self.conn.clone();
        let raw: bool = conn
            .sismember(session_revoked_key(), id.to_string())
            .await
            .map_err(map_redis)?;

        println!("is_revoked: {}", raw);
        Ok(raw)
    }
}
