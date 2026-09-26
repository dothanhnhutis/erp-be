use shared::config::{AppConfig, Environment};
use tracing_appender::{
    non_blocking::{NonBlockingBuilder, WorkerGuard},
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{
    EnvFilter, Layer, filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

/// Giữ sống đến hết main để flush log file
#[must_use = "drop guard sẽ làm mất log trong file"]
pub struct LoggerGuard {
    _file: Option<WorkerGuard>,
}

pub fn init(cfg: &AppConfig) -> anyhow::Result<LoggerGuard> {
    let filter = || EnvFilter::try_new(&cfg.log_filter);

    let (dev_stdout, prod_stdout, prod_file, file_guard) = match cfg.app_env {
        Environment::Dev => (
            Some(fmt::layer().pretty().with_filter(filter()?)),
            None,
            None,
            None,
        ),
        Environment::Prod => {
            let appender = RollingFileAppender::builder()
                .rotation(Rotation::DAILY)
                .filename_prefix(&cfg.crate_name)
                .filename_suffix("log")
                .max_log_files(cfg.log_max_file)
                .build(&cfg.log_dir)?;

            // lossy(false): buffer đầy thì chờ thay vì bỏ log
            let (writer, guard) = NonBlockingBuilder::default().lossy(false).finish(appender);

            (
                None,
                // file JSON là log chính; stdout chỉ WARN+ để không ghi trùng
                Some(
                    fmt::layer()
                        .compact()
                        .with_ansi(false)
                        .with_filter(LevelFilter::WARN),
                ),
                Some(
                    fmt::layer()
                        .json()
                        .with_writer(writer)
                        .with_current_span(true)
                        .with_span_list(false)
                        .with_filter(filter()?),
                ),
                Some(guard),
            )
        }
    };

    tracing_subscriber::registry()
        .with(dev_stdout)
        .with(prod_stdout)
        .with(prod_file)
        .try_init()?;

    // release dùng panic = "abort": ghi dấu vết panic vào log trước khi process dừng
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!(panic = %info, "panic");
        default_hook(info);
    }));

    Ok(LoggerGuard { _file: file_guard })
}

// pub fn init_tracing() -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
//     let default_filter = format!(
//         "info,{}=debug,tower_http=debug,sqlx=warn",
//         env!("CARGO_CRATE_NAME")
//     );
//     let make_filter =
//         || EnvFilter::try_from_default_env().unwrap_or_else(|_| default_filter.clone().into());

//     // File xoay vòng theo ngày: logs/app.2026-09-25.log, giữ tối đa 7 file
//     let file_appender = RollingFileAppender::builder()
//         .rotation(Rotation::DAILY)
//         .filename_prefix("erp-api")
//         .filename_suffix("log")
//         .max_log_files(7)
//         .build("logs")?;
//     let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

//     let terminal_layer = fmt::layer()
//         .pretty() // hoặc bỏ dòng này để dùng format compact mặc định
//         .with_filter(make_filter());

//     let file_layer = fmt::layer()
//         .json()
//         .with_writer(file_writer)
//         .with_ansi(false)
//         .with_current_span(true)
//         .with_span_list(false)
//         .with_filter(make_filter());

//     tracing_subscriber::registry()
//         .with(terminal_layer)
//         .with(file_layer)
//         .init();

//     Ok(guard)
// }
