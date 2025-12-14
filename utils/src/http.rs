use reqwest::header::HeaderMap;
use reqwest::{Client, ClientBuilder};
use std::time::Duration;
use tracing::warn;

pub struct HttpClient {
	timeout_secs: u64,
}

impl HttpClient {
	pub fn new(timeout_secs: u64) -> Self {
		Self { timeout_secs }
	}

	pub fn build_client(&self) -> Client {
		self.build_inner(self.client_builder())
	}

	pub fn build_with_headers(&self, header: HeaderMap) -> Client {
		let builder = self.client_builder().default_headers(header);
		self.build_inner(builder)
	}

	fn client_builder(&self) -> ClientBuilder {
		Client::builder().timeout(Duration::from_secs(self.timeout_secs))
	}

	fn build_inner(&self, builder: ClientBuilder) -> Client {
		builder.build().unwrap_or_else(|err| {
			warn!("Build http client error: {:?}", err);
			Client::new()
		})
	}
}

impl Default for HttpClient {
	fn default() -> Self {
		Self { timeout_secs: 30 }
	}
}
