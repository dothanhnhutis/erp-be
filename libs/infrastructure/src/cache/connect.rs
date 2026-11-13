use redis::aio::ConnectionManager;

pub async fn init_redis(redis_url: &str) -> Result<ConnectionManager, redis::RedisError> {
    let client = redis::Client::open(redis_url)?;
    let conn = client.get_connection_manager().await?;
    Ok(conn)
}

pub fn session_key(user_id: &str, sid: &str) -> String {
    format!("session:{user_id}:{sid}")
}

pub fn user_key(user_id: uuid::Uuid) -> String {
    format!("user_sessions:{user_id}")
}
