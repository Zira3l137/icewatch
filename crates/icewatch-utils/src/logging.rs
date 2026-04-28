use std::{
    fs::File,
    io::Write,
    path::Path,
    sync::{Arc, Mutex, OnceLock},
};

use anyhow::Result;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

static CURRENT_LOG_LEVEL: OnceLock<LevelFilter> = OnceLock::new();

/// Wrapper to make Arc<Mutex<File>> work with tracing's MakeWriter trait
struct MutexWriter(Arc<Mutex<File>>);

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for MutexWriter {
    type Writer = MutexWriterGuard;

    fn make_writer(&'a self) -> Self::Writer {
        MutexWriterGuard(self.0.clone())
    }
}

struct MutexWriterGuard(Arc<Mutex<File>>);

impl Write for MutexWriterGuard {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.lock().unwrap().flush()
    }
}

pub fn get_log_level() -> LevelFilter {
    CURRENT_LOG_LEVEL.get().copied().unwrap_or(LevelFilter::INFO)
}

/// Sets up a workspace-scoped logger with optional file output.
///
/// # Arguments
/// * `level` - Optional log level filter. If `None`, reads from `RUST_LOG` environment variable.
/// * `file` - Optional file path to write logs to. If `Some`, logs are appended to the file.
///
/// # Environment Variables
/// * `RUST_LOG` - Used when `level` is `None` to determine log level
/// * `WORKSPACE_NAME` - Required. Prefix to filter workspace packages (from `.cargo/config.toml`)
pub fn setup_logger<P: AsRef<Path>>(level: Option<LevelFilter>, file: Option<P>) -> Result<()> {
    let workspace_name = env!("WORKSPACE_NAME");

    let base_level = level.unwrap_or_else(|| {
        std::env::var("RUST_LOG")
            .ok()
            .and_then(|s| s.parse::<LevelFilter>().ok())
            .unwrap_or(LevelFilter::INFO)
    });

    CURRENT_LOG_LEVEL.set(base_level).ok();

    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::OFF.into())
        .parse(format!("{}={}", workspace_name, base_level))?;

    let timer = fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S".to_owned());

    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_timer(timer.clone());

    if let Some(file_path) = file {
        let file_path = file_path.as_ref();

        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = std::fs::OpenOptions::new().create(true).append(true).open(file_path)?;
        let writer = MutexWriter(Arc::new(Mutex::new(file)));

        let file_layer = fmt::layer()
            .with_writer(writer)
            .with_timer(timer)
            .with_ansi(false)
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true);

        tracing_subscriber::registry().with(filter).with(console_layer).with(file_layer).init();
    } else {
        tracing_subscriber::registry().with(filter).with(console_layer).init();
    }

    Ok(())
}
