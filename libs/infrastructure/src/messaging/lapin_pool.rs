use deadpool::managed::{Manager, Metrics, Pool, RecycleError, RecycleResult};
use lapin::{
    Channel, Connection, ConnectionProperties, ExchangeKind, options::ConfirmSelectOptions,
    options::ExchangeDeclareOptions, types::FieldTable,
};
use shared::messaging::EMAIL_EXCHANGE;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Giữ 1 connection, tự dựng lại khi hỏng. Channel lấy từ đây.
pub struct ChannelManager {
    uri: String,
    conn: Mutex<Option<Arc<Connection>>>, // thêm Arc
}

impl ChannelManager {
    pub fn new(uri: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            conn: Mutex::new(None),
        }
    }

    async fn connection(&self) -> Result<Arc<Connection>, lapin::Error> {
        let mut guard = self.conn.lock().await;
        if let Some(c) = guard.as_ref()
            && c.status().connected()
        {
            return Ok(c.clone()); // clone Arc, rẻ
        }
        // connection chưa có / đã chết → dựng lại (chỗ "reconnect")
        let c = Arc::new(Connection::connect(&self.uri, ConnectionProperties::default()).await?);
        *guard = Some(c.clone());
        Ok(c)
    }
}

impl Manager for ChannelManager {
    type Type = Channel;
    type Error = lapin::Error;

    async fn create(&self) -> Result<Channel, lapin::Error> {
        let conn = self.connection().await?;
        let ch = conn.create_channel().await?;
        ch.confirm_select(ConfirmSelectOptions::default()).await?; // bật publisher confirm
        Ok(ch)
    }

    async fn recycle(&self, ch: &mut Channel, _: &Metrics) -> RecycleResult<lapin::Error> {
        if ch.status().connected() {
            Ok(()) // tái sử dụng
        } else {
            Err(RecycleError::message("channel closed")) // pool tự loại, gọi create() lại
        }
    }
}

pub type ChannelPool = Pool<ChannelManager>;

pub fn build_channel_pool(uri: &str, max: usize) -> ChannelPool {
    Pool::builder(ChannelManager::new(uri))
        .max_size(max) // 4–8 như bạn muốn
        .build()
        .expect("build channel pool")
}

/// Declare exchange email (idempotent) — gọi lúc api khởi động để publish không
/// phụ thuộc việc worker đã chạy hay chưa. Giấu lapin khỏi tầng api.
pub async fn declare_email_topology(
    pool: &ChannelPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ch = pool.get().await?;
    ch.exchange_declare(
        EMAIL_EXCHANGE.into(),
        ExchangeKind::Direct,
        ExchangeDeclareOptions {
            durable: true,
            ..Default::default()
        },
        FieldTable::default(),
    )
    .await?;
    Ok(())
}
