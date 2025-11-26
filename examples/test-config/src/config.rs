use base_infra::logger::Logger;
use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TestAppConfig {
    #[serde(rename = "log")]
    logger: Logger,
}
impl TestAppConfig {
    pub fn logger(&self) -> &Logger {
        &self.logger
    }
}
