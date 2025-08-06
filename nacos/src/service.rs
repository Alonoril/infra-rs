use crate::types::{ConfigNotifySender, NacosServerConfig};
use nacos_sdk::api::config::{
	ConfigChangeListener, ConfigResponse, ConfigService, ConfigServiceBuilder,
};
use nacos_sdk::api::props::ClientProps;
use std::sync::Arc;
use tracing::{error, info};

type DynConfigService = Box<dyn ConfigService + Send + Sync>;

pub struct NacosConfigService {
	pub service: DynConfigService,
	pub config: NacosServerConfig,
}

impl NacosConfigService {
	pub(crate) fn new(config: NacosServerConfig) -> anyhow::Result<Self> {
		let config_service = ConfigServiceBuilder::new((&config).into())
			.enable_auth_plugin_http()
			.build()?;
		let service: DynConfigService = Box::new(config_service);
		Ok(Self { service, config })
	}

	pub(crate) async fn add_listener(&self, sender: ConfigNotifySender) -> anyhow::Result<()> {
		let data_id = self.config.data_id.clone();
		let group_name = self.config.group_name.clone();

		let listener = Arc::new(NacosConfigListener::new(sender));
		self.service
			.add_listener(data_id, group_name, listener)
			.await?;
		info!("add listener success");
		Ok(())
	}

	pub(crate) async fn get_config(&self) -> anyhow::Result<ConfigResponse> {
		let data_id = self.config.data_id.to_string();
		let group_name = self.config.group_name.to_string();
		Ok(self.service.get_config(data_id, group_name).await?)
	}
}

#[derive(Clone)]
struct NacosConfigListener {
	sender: ConfigNotifySender,
}

impl NacosConfigListener {
	pub fn new(sender: ConfigNotifySender) -> Self {
		Self { sender }
	}
}

impl ConfigChangeListener for NacosConfigListener {
	fn notify(&self, resp: ConfigResponse) {
		let ct = resp.content_type();
		info!("listen content_type[{ct}] remote config={resp}");

		let this = self.clone();
		if let Err(e) = this.sender.send(resp) {
			error!("Notify ConfigChange error: {e}");
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_get_config() -> anyhow::Result<()> {
		let cfg = NacosServerConfig::new(
			"127.0.0.1:8848",
			"nacos",
			"nacos",
			"",
			"swap",
			// "eds-swap.yaml",
			"eds-swap",
		);
		let nacos = NacosConfigService::new(cfg)?;
		let config_resp = nacos
			.service
			.get_config("eds-swap".to_string(), "swap".to_string())
			.await?;
		println!("get the config {}", config_resp);
		println!("config content_type={}", config_resp.content_type());

		Ok(())
	}
}
