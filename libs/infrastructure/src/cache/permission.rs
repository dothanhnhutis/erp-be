use domain::cache::{CacheError, PermissionCache};
use redis::{AsyncCommands, aio::ConnectionManager};
use uuid::Uuid;

use crate::persistence::error::{map_json, map_redis};

const PERM_VER_KEY: &str = "perm:ver";

fn perm_key(ver: u64, user_id: Uuid) -> String {
    format!("perm:v{ver}:user:{user_id}")
}

#[derive(Clone)]
pub struct RedisPermissionCache {
    conn: ConnectionManager,
}

impl RedisPermissionCache {
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }

    async fn current_version(conn: &mut ConnectionManager) -> Result<u64, CacheError> {
        let ver: Option<u64> = conn.get(PERM_VER_KEY).await.map_err(map_redis)?;
        Ok(ver.unwrap_or(0))
    }
}

impl PermissionCache for RedisPermissionCache {
    async fn get(&self, user_id: Uuid) -> Result<(u64, Option<Vec<String>>), CacheError> {
        let mut conn = self.conn.clone();
        let ver = Self::current_version(&mut conn).await?;
        let raw: Option<String> = conn.get(perm_key(ver, user_id)).await.map_err(map_redis)?;
        let codes = raw
            .map(|s| serde_json::from_str::<Vec<String>>(&s))
            .transpose()
            .map_err(map_json)?;
        Ok((ver, codes))
    }

    async fn put(
        &self,
        version: u64,
        user_id: Uuid,
        codes: &[String],
        ttl_secs: u64,
    ) -> Result<(), CacheError> {
        let mut conn = self.conn.clone();
        // Lưu JSON (kể cả "[]") để cache được user không có quyền nào.
        let json = serde_json::to_string(codes).map_err(map_json)?;
        let _: () = conn
            .set_ex(perm_key(version, user_id), json, ttl_secs.max(1))
            .await
            .map_err(map_redis)?;
        Ok(())
    }

    async fn invalidate_user(&self, user_id: Uuid) -> Result<(), CacheError> {
        let mut conn = self.conn.clone();
        let ver = Self::current_version(&mut conn).await?;
        let _: () = conn.del(perm_key(ver, user_id)).await.map_err(map_redis)?;
        Ok(())
    }

    async fn bump_version(&self) -> Result<(), CacheError> {
        let mut conn = self.conn.clone();
        let _: u64 = conn.incr(PERM_VER_KEY, 1).await.map_err(map_redis)?;
        Ok(())
    }
}
