use nacos_sdk::api::config::ConfigResponse;
use nacos_sdk::api::props::ClientProps;
use std::sync::mpsc;

pub(crate) type ConfigNotifySender = mpsc::Sender<ConfigResponse>;

#[derive(Debug, Clone)]
pub struct NacosServer {
	pub server_addr: String,
	pub username: String,
	pub password: String,
}

impl NacosServer {
	pub fn new(
		server_addr: impl Into<String>,
		username: impl Into<String>,
		password: impl Into<String>,
	) -> Self {
		Self {
			server_addr: server_addr.into(),
			username: username.into(),
			password: password.into(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct GroupKey {
	pub namespace: String,
	pub group_name: String,
	pub data_id: String,
}
impl GroupKey {
	pub fn new(
		namespace: impl Into<String>,
		group_name: impl Into<String>,
		data_id: impl Into<String>,
	) -> Self {
		Self {
			namespace: namespace.into(),
			group_name: group_name.into(),
			data_id: data_id.into(),
		}
	}
}

#[derive(Debug, Clone)]
pub(crate) struct NacosServerConfig {
	pub server_addr: String,
	pub username: String,
	pub password: String,
	pub namespace: String,
	pub group_name: String,
	pub data_id: String,
}

impl NacosServerConfig {
	pub(crate) fn new(
		server_addr: impl Into<String>,
		username: impl Into<String>,
		password: impl Into<String>,
		namespace: impl Into<String>,
		group_name: impl Into<String>,
		data_id: impl Into<String>,
	) -> Self {
		Self {
			server_addr: server_addr.into(),
			username: username.into(),
			password: password.into(),
			namespace: namespace.into(),
			group_name: group_name.into(),
			data_id: data_id.into(),
		}
	}
}

impl From<&NacosServerConfig> for ClientProps {
	fn from(config: &NacosServerConfig) -> Self {
		ClientProps::new()
			.server_addr(&*config.server_addr)
			.namespace(&*config.namespace)
			.auth_username(&*config.username)
			.auth_password(&*config.password)
	}
}

impl From<(NacosServer, GroupKey)> for NacosServerConfig {
	fn from((server, group): (NacosServer, GroupKey)) -> Self {
		Self {
			server_addr: server.server_addr,
			username: server.username,
			password: server.password,
			namespace: group.namespace,
			group_name: group.group_name,
			data_id: group.data_id,
		}
	}
}
