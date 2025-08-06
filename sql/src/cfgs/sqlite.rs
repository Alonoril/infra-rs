use crate::cfgs::DbCfgTrait;
use anyhow::Context;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

pub static DB_URL_PREFIX: &str = "sqlite://";
// pub static DB_URL_SUFFIX: &str = "?mode=rwc";
pub static DB_URL_SUFFIX: &str = "";
pub static DB_FILE: &str = "dex_amm_bot.db3";

// let db_url = format!("{}{}{}", url_prefix, db_file, url_suffix);

#[derive(Debug, Clone, Deserialize)]
pub struct DbConfig {
    pub app_dir: PathBuf,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
    pub run_migrations: bool,
}

impl DbConfig {
    pub fn new(app_dir: PathBuf) -> Self {
        Self {
            app_dir,
            ..Default::default()
        }
    }

    pub fn db_url(&self) -> anyhow::Result<String> {
        let db_file = self.app_dir.join("db");
        if !db_file.exists() {
            fs::create_dir_all(db_file.clone()).context("Failed to create db directory")?;
        }

        let db_file = db_file.join(DB_FILE);
        if !db_file.exists() {
            fs::File::create(db_file.clone()).context("Failed to create db file")?;
        }

        Ok(format!(
            "{}{}{}",
            DB_URL_PREFIX,
            db_file.display(),
            DB_URL_SUFFIX
        ))
    }
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            app_dir: PathBuf::from("../data/"),
            max_connections: 10,
            min_connections: 10,
            connect_timeout_secs: 30,
            idle_timeout_secs: 1800,
            max_lifetime_secs: 3600,
            run_migrations: true,
        }
    }
}

impl DbCfgTrait for DbConfig {
    fn db_url(&self) -> String {
        self.db_url().unwrap()
    }

    fn debug_db_url(&self) -> String {
        self.db_url().unwrap()
    }

    fn max_conns(&self) -> u32 {
        self.max_connections
    }

    fn min_conns(&self) -> u32 {
        self.min_connections
    }

    fn conn_timeout_secs(&self) -> u64 {
        self.connect_timeout_secs
    }

    fn idle_timeout_secs(&self) -> u64 {
        self.idle_timeout_secs
    }

    fn max_lifetime_secs(&self) -> u64 {
        self.max_lifetime_secs
    }
}
