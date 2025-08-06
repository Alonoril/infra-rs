use crate::app_err;
use crate::result::{AppResult, SysErr};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::Level;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RtEnv {
    Development,
    Production,
}
impl RtEnv {
    pub fn is_dev(&self) -> bool {
        matches!(self, Self::Development)
    }
    pub fn is_prod(&self) -> bool {
        matches!(self, Self::Production)
    }
}
impl Default for RtEnv {
    fn default() -> Self {
        RtEnv::Production
    }
}

#[derive(Clone, Debug)]
pub struct LocalConfig {
    pub rt_env: RtEnv,
    /// log level
    pub log_level: Option<Level>,
    pub config_path: Option<PathBuf>,
}

impl LocalConfig {
    pub fn log_level(&self) -> Level {
        self.log_level.unwrap_or(Level::INFO)
    }

    pub fn config_path(&self) -> AppResult<PathBuf> {
        let path = self
            .config_path
            .clone()
            .ok_or(app_err!(&SysErr::NoCfgFile))?;
        // .unwrap_or_else(|| PathBuf::from("config.yaml"))
        // .canonicalize()
        // .map_err(|e| anyhow::anyhow!("Invalid config path: {}", e))?;
        Ok(path)
    }
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            rt_env: RtEnv::Development,
            log_level: Some(Level::DEBUG),
            config_path: Some(PathBuf::from("./configs/swap-config.yaml")),
        }
    }
}
