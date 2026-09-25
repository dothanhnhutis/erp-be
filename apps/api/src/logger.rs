use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing() -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    let default_filter = format!(
        "info,{}=debug,tower_http=debug,sqlx=warn",
        env!("CARGO_CRATE_NAME")
    );
    let make_filter =
        || EnvFilter::try_from_default_env().unwrap_or_else(|_| default_filter.clone().into());

    // File xoay vòng theo ngày: logs/app.2026-09-25.log, giữ tối đa 7 file
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("erp-api")
        .filename_suffix("log")
        .max_log_files(7)
        .build("logs")?;
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let terminal_layer = fmt::layer()
        .pretty() // hoặc bỏ dòng này để dùng format compact mặc định
        .with_filter(make_filter());

    let file_layer = fmt::layer()
        .json()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_current_span(true)
        .with_span_list(false)
        .with_filter(make_filter());

    tracing_subscriber::registry()
        .with(terminal_layer)
        .with(file_layer)
        .init();

    Ok(guard)
}
