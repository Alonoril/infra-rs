//! Initialize logger.

use crate::config::{LocalConfig, RtEnv};
use serde::Deserialize;
use std::path::PathBuf;
use std::{panic, thread};
use tracing::{error, level_filters::LevelFilter};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, registry};

/// Initialize logger (tracing and panic hook).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Logger {
    pub path: PathBuf,
    pub directives: Vec<String>,
}

impl Logger {
    pub fn with_path(self, path: PathBuf) -> Self {
        Self { path, ..self }
    }

    pub fn init(&self, app_args: &LocalConfig) -> WorkerGuard {
        let app_env: RtEnv = app_args.rt_env;
        let console_logger = std::io::stdout();

        let (non_blocking, guard) = match app_env {
            RtEnv::Development => tracing_appender::non_blocking(console_logger),
            RtEnv::Production => {
                let dir = self.path.join("logs");
                let file_logger = rolling::daily(dir, "default.log");
                tracing_appender::non_blocking(file_logger)
            }
        };

        let layer = Layer::new()
            .with_line_number(true)
            .with_thread_names(true)
            .with_thread_ids(true)
            .with_ansi(self.is_ansi(app_args))
            .with_writer(non_blocking);

        let layered = registry()
            // .with(max_level)
            .with(self.build_env_filter(app_args))
            .with(layer);

        layered.init();
        // init panic hook
        self.panic_hook();

        guard
    }

    fn build_env_filter(&self, app_args: &LocalConfig) -> EnvFilter {
        let app_env: RtEnv = app_args.rt_env;
        let max_level = match app_args.log_level {
            Some(level) => level.into(),
            None => match app_env {
                RtEnv::Development => LevelFilter::TRACE,
                RtEnv::Production => LevelFilter::DEBUG,
            },
        };

        let mut env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(max_level.to_string()));
        for directive in &self.directives {
            env_filter = env_filter.add_directive(directive.parse().expect("invalid directive"));
        }

        env_filter
    }

    fn panic_hook(&self) {
        // catch panic and log them using tracing instead of default output to StdErr
        panic::set_hook(Box::new(|info| {
            let thread = thread::current();
            let thread = thread.name().unwrap_or("unknown");

            let msg = match info.payload().downcast_ref::<&'static str>() {
                Some(s) => *s,
                None => match info.payload().downcast_ref::<String>() {
                    Some(s) => &**s,
                    None => "Box<Any>",
                },
            };

            let backtrace = backtrace::Backtrace::new();

            match info.location() {
                Some(location) => {
                    // without backtrace
                    if msg.starts_with("notrace - ") {
                        error!(
                            target: "panic", "thread '{}' panicked at '{}': {}:{}",
                            thread,
                            msg.replace("notrace - ", ""),
                            location.file(),
                            location.line()
                        );
                    }
                    // with backtrace
                    else {
                        error!(
                            target: "panic", "thread '{}' panicked at '{}': {}:{}\n{:?}",
                            thread,
                            msg,
                            location.file(),
                            location.line(),
                            backtrace
                        );
                    }
                }
                None => {
                    // without backtrace
                    if msg.starts_with("notrace - ") {
                        error!(
                            target: "panic", "thread '{}' panicked at '{}'",
                            thread,
                            msg.replace("notrace - ", ""),
                        );
                    }
                    // with backtrace
                    else {
                        error!(
                            target: "panic", "thread '{}' panicked at '{}'\n{:?}",
                            thread,
                            msg,
                            backtrace
                        );
                    }
                }
            }
        }));
    }

    fn is_ansi(&self, args: &LocalConfig) -> bool {
        match args.rt_env {
            RtEnv::Development => true,
            RtEnv::Production => false,
        }
    }
}
