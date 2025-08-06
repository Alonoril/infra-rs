use crate::error::DBErr;
use base_infra::map_err;
use base_infra::result::AppResult;
use sea_orm::Database as SeaDatabase;
use sea_orm::{ConnectOptions, DatabaseConnection};
use std::time::Duration;
use tracing::info;

pub mod cfgs;
pub mod error;
pub mod sea_ext;
pub mod utils;
pub mod macros;

use crate::cfgs::DbCfgTrait;

#[async_trait::async_trait]
pub trait DatabaseTrait<T, Cfg: DbCfgTrait + Sync + Send> {
    async fn setup(cfg: &Cfg) -> AppResult<T>;

    async fn connect(cfg: &Cfg) -> AppResult<DatabaseConnection> {
        let mut opt = ConnectOptions::new(cfg.db_url());
        opt.max_connections(cfg.max_conns())
            .min_connections(cfg.min_conns())
            .connect_timeout(Duration::from_secs(cfg.conn_timeout_secs()))
            .idle_timeout(Duration::from_secs(cfg.idle_timeout_secs()))
            .max_lifetime(Duration::from_secs(cfg.max_lifetime_secs()));

        let pool = SeaDatabase::connect(opt)
            .await
            .map_err(map_err!(&DBErr::InitDbPoolErr, cfg.debug_db_url()))?;

        info!("connected to database，url: {}", cfg.debug_db_url());
        Ok(pool)
    }
}
