use figment::Figment;
use figment::providers::{Format, Json, Toml, Yaml};
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Config parse error: {0}")]
	ConfigParseErr(#[from] figment::Error),
	#[error("Not support config content type: {0}")]
	ContentTypeErr(String),
}

/// Content's Type; e. g. yaml,toml,json
#[derive(Debug, Clone, Copy)]
pub enum ConfigType {
	Yaml,
	Toml,
	Json,
}

impl FromStr for ConfigType {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"yaml" | "YAML" => Ok(ConfigType::Yaml),
			"toml" | "TOML" => Ok(ConfigType::Toml),
			"json" | "JSON" => Ok(ConfigType::Json),
			_ => Err(Error::ContentTypeErr(s.to_string())),
		}
	}
}

pub trait ConfigExt
where
	Self: for<'de> Deserialize<'de>,
{
	fn parse(ct: ConfigType, content: &str) -> anyhow::Result<Self> {
		let figment = match ct {
			ConfigType::Yaml => Figment::new().merge(Yaml::string(content)),
			ConfigType::Toml => Figment::new().merge(Toml::string(content)),
			ConfigType::Json => Figment::new().merge(Json::string(content)),
		};
		Ok(figment.extract().map_err(Error::ConfigParseErr)?)
	}
}

impl<T> ConfigExt for T where T: for<'de> Deserialize<'de> {}
