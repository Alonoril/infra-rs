use cfg_nacos::client::NacosConfigClient;
use cfg_nacos::types::{GroupKey, NacosServer};
use serde::Deserialize;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
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

#[derive(Clone)]
pub struct BizService {
	pub nacos_client: Arc<NacosConfigClient<AppConfig>>,
}

async fn auto_update_config() -> anyhow::Result<()> {
	let svr = NacosServer::new("127.0.0.1:8848", "nacos", "nacos");
	// "eds-swap.yaml",
	let grp = GroupKey::new("swap-test1", "DEFAULT_GROUP", "eds-swap.toml");
	let client = NacosConfigClient::new(svr, grp, AppConfig::default()).await?;

	tokio::spawn(async move {
		let biz_service = BizService {
			nacos_client: client,
		};

		loop {
			println!("remote_config: {:?}", biz_service.nacos_client.get_config());
			time::sleep(Duration::from_secs(3)).await;
		}
	});

	Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	auto_update_config().await?;

	let term = Arc::new(AtomicBool::new(false));
	while !term.load(Ordering::Acquire) {
		thread::park();
	}

	Ok(())
}
