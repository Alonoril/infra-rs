use nacos_infra::client::NacosConfigClient;
use nacos_infra::types::{GroupKey, NacosServer};
use serde::Deserialize;
use std::time::Duration;
use tokio::time;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct WebConfig {
	pub address: String,
	pub port: u16,
	pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AppConfig {
	#[serde(rename = "web")]
	pub api_config: WebConfig,
}

async fn load_config() -> anyhow::Result<()> {
	let svr = NacosServer::new("127.0.0.1:8848", "nacos", "nacos");
	// "eds-swap.yaml",
	let grp = GroupKey::new("swap-test1", "DEFAULT_GROUP", "eds-swap.toml");
	let client = NacosConfigClient::new(svr, grp, AppConfig::default()).await?;

	loop {
		let remote_config = client.get_config();
		println!("remote_config: {:?}", remote_config);

		time::sleep(Duration::from_secs(3)).await;
	}
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	load_config().await?;

	Ok(())
}
