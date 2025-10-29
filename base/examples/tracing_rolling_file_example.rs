use base_infra::config::{RtEnv};
use std::path::PathBuf;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_rolling_file::{RollingConditionBase, RollingFileAppenderBase};
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, registry};

fn main() {
    let _guard = init();

    loop {
        tracing::info!("Hello, world!");
        tracing::error!("Hello, world!");
        tracing::warn!("Hello, world!");
        tracing::debug!("Hello, world!");
        tracing::trace!("Hello, world!");

        std::thread::sleep(std::time::Duration::from_secs(20));
    }
}

pub fn init() -> WorkerGuard {
    let console_logger = std::io::stdout();
    let app_env = RtEnv::Production;
    let path = PathBuf::from("./");
    let is_ansi = app_env == RtEnv::Development;

    let file_appender =
        RollingFileAppenderBase::new("./log/test.log", RollingConditionBase::new().minutely(), 9)
            .unwrap();
    let (non_blocking, guard) = file_appender.get_non_blocking_appender();

    let layer = Layer::new()
        .with_line_number(true)
        .with_thread_names(true)
        .with_thread_ids(true)
        .with_ansi(is_ansi)
        .with_writer(non_blocking);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(LevelFilter::TRACE.to_string()));
    let layered = registry().with(env_filter).with(layer);

    layered.init();

    guard
}
