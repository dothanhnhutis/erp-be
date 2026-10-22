use shared::messaging::EmailJob;

use crate::errors::AppError;

pub trait EmailPublisher: Send + Sync {
    fn publish(&self, job: EmailJob) -> impl Future<Output = Result<(), AppError>> + Send;
}
