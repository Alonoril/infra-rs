use base_infra::config::{LocalConfig, RtEnv};
pub use clap::Parser;
use std::path::PathBuf;
use tracing::Level;

#[derive(clap::ValueEnum, Clone, Debug, Copy)]
pub enum AppEnv {
	Development,
	Production,
}

#[derive(clap::Parser)]
pub struct AppArgs {
	#[clap(long, env, value_enum)]
	pub app_env: AppEnv,
	/// log level
	#[clap(long, env, default_value = "INFO")]
	#[arg(value_parser = parse_level)]
	pub log_level: Option<Level>,
	/// Path to application configuration file (or template for local test mode).
	#[clap(long, env, value_parser)]
	pub config: Option<PathBuf>,
	/// Git commit  hash
	#[clap(long, short = 'c', value_parser)]
	pub commit: bool,
}

fn parse_level(level: &str) -> anyhow::Result<Level> {
	let level: Level = level
		.parse()
		.map_err(|e| anyhow::anyhow!("Invalid log level: {:?}", e))?;
	Ok(level)
}

impl From<AppArgs> for LocalConfig {
	fn from(value: AppArgs) -> Self {
		let env: RtEnv = match value.app_env {
			AppEnv::Development => RtEnv::Development,
			AppEnv::Production => RtEnv::Production,
		};

		Self {
			rt_env: env,
			log_level: value.log_level,
			config_path: value.config,
		}
	}
}
