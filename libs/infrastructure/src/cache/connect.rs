use redis::aio::ConnectionManager;

pub async fn init_redis(redis_url: &str) -> Result<ConnectionManager, redis::RedisError> {
    let client = redis::Client::open(redis_url)?;
    let conn = client.get_connection_manager().await?;
    Ok(conn)
}
