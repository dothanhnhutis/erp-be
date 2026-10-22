use super::lapin_pool::ChannelPool;
use application::{errors::AppError, ports::EmailPublisher};
use lapin::{BasicProperties, options::BasicPublishOptions};
use shared::messaging::EmailJob;

#[derive(Clone)]
pub struct LapinEmailPublisher {
    pool: ChannelPool, // Pool là Arc bên trong → Clone rẻ
    exchange: String,
    routing_key: String,
}

impl LapinEmailPublisher {
    pub fn new(
        pool: ChannelPool,
        exchange: impl Into<String>,
        routing_key: impl Into<String>,
    ) -> Self {
        Self {
            pool,
            exchange: exchange.into(),
            routing_key: routing_key.into(),
        }
    }
}

impl EmailPublisher for LapinEmailPublisher {
    async fn publish(&self, job: EmailJob) -> Result<(), AppError> {
        let payload = serde_json::to_vec(&job).map_err(|e| AppError::Internal(e.to_string()))?;

        let ch = self
            .pool
            .get()
            .await // lấy channel (tự reconnect nếu cần)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        ch.basic_publish(
            self.exchange.as_str().into(),
            self.routing_key.as_str().into(),
            BasicPublishOptions::default(),
            &payload,
            BasicProperties::default().with_delivery_mode(2), // 2 = persistent, không mất khi broker restart
        )
        .await
        .map_err(|e| AppError::Internal(e.to_string()))? // gửi frame
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?; // CHỜ broker ack (confirm)
        Ok(())
    }
}
