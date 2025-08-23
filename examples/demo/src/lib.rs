use crate::config::TestAppConfig;
use base_infra::WorkerGuard;
use base_infra::config::{ConfigExt, LocalConfig};
use clap::Parser;
use cli_infra::AppArgs;
use std::sync::Arc;
use tracing::debug;

pub mod config;

pub async fn setup_logger() -> anyhow::Result<(Arc<TestAppConfig>, WorkerGuard)> {
    dotenvy::dotenv().ok();
    let local_cfg: LocalConfig = AppArgs::parse().into();
    eprintln!(">>>cli config: {local_cfg:?}");

    let app_cfg = get_config_client_test(&local_cfg).await?;

    let _guard = app_cfg.logger().init(&local_cfg);
    debug!("AppConfig info: {app_cfg:?}");

    Ok((app_cfg, _guard))
}

pub async fn get_config_client_test(local_cfg: &LocalConfig) -> anyhow::Result<Arc<TestAppConfig>> {
    let app_cfg = TestAppConfig::load(local_cfg.config_path()?)?;
    Ok(Arc::new(app_cfg))
}
