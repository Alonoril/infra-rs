use crate::core::ConfigExt;
use crate::service::NacosConfigService;
use crate::types::{GroupKey, NacosServer};
use arc_swap::ArcSwap;
use base_infra::runtimes::Tokio;
use nacos_sdk::api::config::ConfigResponse;
use serde::de::DeserializeOwned;
use std::sync::{Arc, mpsc};
use tracing::info;

pub trait GlobalConfigClient<C>
where
	C: DeserializeOwned + Send + Sync + Clone + 'static,
{
	fn get(&self) -> Arc<C>;

	fn cache(&mut self, config: C);
}

pub struct NacosConfigClient<C> {
	config_service: NacosConfigService,
	cached_config: ArcSwap<C>,
}

impl<C> NacosConfigClient<C>
where
	C: DeserializeOwned + Send + Sync + Clone + 'static,
{
	pub async fn new(svr: NacosServer, group: GroupKey, config: C) -> anyhow::Result<Arc<Self>> {
		let client = Self {
			config_service: NacosConfigService::new((svr, group).into())?,
			cached_config: ArcSwap::new(Arc::new(config)),
		};

		// init remote config
		client.get_remote_config().await?;
		eprintln!("[EndlessNacosClient] NacosConfig client init success");

		let client = Arc::new(client);
		// start listen
		listen(client.clone()).await?;
		Ok(client)
	}

	pub fn get_config(&self) -> Arc<C> {
		self.cached_config.load().clone()
	}

	pub async fn get_remote_config(&self) -> anyhow::Result<Arc<C>> {
		let resp = self.config_service.get_config().await?;
		let config = parse(resp)?;
		self.cached_config.store(Arc::new(config));

		Ok(self.get_config())
	}
}

impl<C> GlobalConfigClient<C> for NacosConfigClient<C>
where
	C: DeserializeOwned + Send + Sync + Clone + 'static,
{
	fn get(&self) -> Arc<C> {
		self.get_config()
	}

	fn cache(&mut self, config: C) {
		self.cached_config.store(Arc::new(config));
	}
}

async fn listen<C>(client: Arc<NacosConfigClient<C>>) -> anyhow::Result<()>
where
	C: DeserializeOwned + Send + Sync + 'static,
{
	let (tx, rx) = mpsc::channel();
	client.config_service.add_listener(tx).await?;

	let client = client.clone();
	Tokio.spawn(async move {
		while let Ok(cr) = rx.recv() {
			client.cached_config.store(Arc::new(parse(cr)?));
			info!("use remote config to update local config success");
		}
		Ok::<(), anyhow::Error>(())
	});
	Ok(())
}

fn parse<C>(resp: ConfigResponse) -> anyhow::Result<C>
where
	C: DeserializeOwned,
{
	let ct = resp.content_type().parse()?;
	C::parse(ct, resp.content())
}
