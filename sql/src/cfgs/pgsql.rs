use crate::cfgs::DbCfgTrait;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

#[derive(Clone, Serialize, Deserialize)]
pub struct DbConfig {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
    pub run_migrations: bool,
}

impl DbConfig {
    pub fn new(
        username: String,
        password: String,
        host: String,
        port: u16,
        database: String,
    ) -> Self {
        Self {
            username,
            password,
            host,
            port,
            database,
            ..Default::default()
        }
    }
}

impl DbCfgTrait for DbConfig {
    fn db_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }

    fn debug_db_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, "*****", self.host, self.port, self.database
        )
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

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            username: "postgres".to_string(),
            password: "postgres".to_string(),
            host: "localhost".to_string(),
            port: 5432,
            database: "postgres".to_string(),
            max_connections: 5,
            min_connections: 0,
            connect_timeout_secs: 5,
            idle_timeout_secs: 30,
            max_lifetime_secs: 3600,
            run_migrations: true,
        }
    }
}

impl Debug for DbConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, "*****", self.host, self.port, self.database
        );
        f.debug_struct("DbConfig")
            .field("database_url", &database_url)
            .field("max_connections", &self.max_connections)
            .field("min_connections", &self.min_connections)
            .field("run_migrations", &self.run_migrations)
            .field("connect_timeout_secs", &self.connect_timeout_secs)
            .field("idle_timeout_secs", &self.idle_timeout_secs)
            .field("max_lifetime_secs", &self.max_lifetime_secs)
            .finish()
    }
}
