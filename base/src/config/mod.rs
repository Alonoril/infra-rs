mod local;

pub use local::*;

use crate::map_err;
use crate::result::{AppResult, SysErr};
use figment::Figment;
use figment::providers::{Env, Format, Toml, Yaml};
use serde::Deserialize;
use std::path::PathBuf;

pub trait ConfigExt
where
	Self: for<'de> Deserialize<'de>,
{
	/// Load the configuration from the file at the value of the args(ENV/cli) `CONFIG`
	/// or `config.yaml` by default, with an overlay provided by environment variables prefixed with
	/// `"APP__"` and split/nested via `"__"`.
	// fn load(path: PathBuf) -> Result<Self, figment::Error> {
	fn load(path: PathBuf) -> AppResult<Self> {
		let config = Figment::new()
			.merge(Toml::string(""))
			.merge(Yaml::string(""))
			.merge(Yaml::file_exact(path))
			.merge(Env::prefixed("APP__").split("__"))
			.extract()
			.map_err(map_err!(&SysErr::ConfigLoadFailed))?;

		Ok(config)
	}
}

impl<T> ConfigExt for T where T: for<'de> Deserialize<'de> {}
