use std::fs::File;
use serde::Deserialize;
use std::io::Read;
use serenity::prelude::TypeMapKey;

#[derive(Deserialize)]
pub struct Config {
	pub general: General,
}

#[derive(Deserialize)]
pub struct General {
	pub discord_token: String,
	pub prefix: String,
}

impl Config {
	pub fn new_from_file(mut file: File) -> Self {
		let mut file_contents = String::new();
		file.read_to_string(&mut file_contents).expect("Couldn't read config file");

		Self::from_str(file_contents.as_str())
	}

	pub fn from_str(config_toml: &str) -> Self {
		log::debug!("Parsing config");
		toml::from_str(config_toml).expect("Malformed toml input")
	}
}

impl TypeMapKey for Config {
	type Value = Config;
}

#[cfg(test)]
mod tests {
	#[test]
	fn test_parse_config() {
		let config_str = r"
		[general]
		discord_token = 'test_token'
		prefix = '-'
		";

		let config = super::Config::from_str(config_str);

		assert_eq!(config.general.discord_token, "test_token");
		assert_eq!(config.general.prefix, "-");
	}
}