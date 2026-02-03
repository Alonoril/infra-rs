use base_infra::logger::Logger;
use serde::Deserialize;
use sql_infra::cfgs::pgsql::DbConfig;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TestAppConfig {
	#[serde(rename = "log")]
	logger: Logger,
	#[serde(rename = "db")]
	pub db_config: DbConfig,
}
impl TestAppConfig {
	pub fn logger(&self) -> &Logger {
		&self.logger
	}
}
