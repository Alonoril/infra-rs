use std::fmt::Debug;

#[cfg(feature = "pgsql")]
pub mod pgsql;
#[cfg(feature = "sqlite")]
pub mod sqlite;

pub trait DbCfgTrait: Default + Debug + Send + Sync {
    fn db_url(&self) -> String;
    fn debug_db_url(&self) -> String;
    fn max_conns(&self) -> u32;
    fn min_conns(&self) -> u32;
    fn conn_timeout_secs(&self) -> u64;
    fn idle_timeout_secs(&self) -> u64;
    fn max_lifetime_secs(&self) -> u64;
    fn run_migrations(&self) -> bool;
}
