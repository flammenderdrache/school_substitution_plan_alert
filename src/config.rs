use std::fs::File;
use serde::Deserialize;
use std::io::Read;
use serenity::prelude::TypeMapKey;
use serenity::model::prelude::UserId;
use std::collections::HashSet;

#[derive(Deserialize)]
pub struct Config {
	pub general: General,
}

#[derive(Deserialize)]
pub struct General {
	pub discord_token: String,
	pub prefix: String,
	pub owners: HashSet<UserId>,
	#[serde(default)]
	pub class_whitelist: HashSet<String>, //Pre defined classes, these are loaded into the whitelist on startup.
}

impl Config {
	pub fn from_file(mut file: File) -> Self {
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
	use std::collections::HashSet;
	use serenity::model::id::UserId;

	#[test]
	fn test_parse_config() {
		let config_str = r"
		[general]
		discord_token = 'test_token'
		prefix = '-'
		owners = [191594115907977225, 276431762815451138, 325704347767799808]
		class_whitelist = [
    		'Class 1',
    		'Class 2',
		]
		";


		let config = super::Config::from_str(config_str);
		let mut owners = HashSet::new();
		owners.insert(UserId::from(191594115907977225));
		owners.insert(UserId::from(276431762815451138));
		owners.insert(UserId::from(325704347767799808));

		let mut classes = HashSet::new();
		classes.insert("Class 1".to_owned());
		classes.insert("Class 2".to_owned());

		assert_eq!(config.general.discord_token, "test_token");
		assert_eq!(config.general.prefix, "-");
		assert_eq!(owners, config.general.owners);
		assert_eq!(classes, config.general.class_whitelist)
	}
}