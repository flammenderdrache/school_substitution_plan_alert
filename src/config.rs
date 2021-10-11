use std::collections::HashSet;
use std::fs::File;
use std::io::Read;

use serde::Deserialize;
use serenity::model::prelude::UserId;
use serenity::prelude::TypeMapKey;

/// This struct holds the other more specific config structs
#[derive(Deserialize)]
pub struct Config {
	pub general: General,
}

/// The struct for general config stuff. More specific functionality, specific functionality like
/// Database related information would go inside a specific database struct
#[derive(Deserialize)]
pub struct General {
	/// No default value, without the token we can't do anything.
	/// The bot refuses to start if the token is missing.
	pub discord_token: String,
	/// The prefix is '~' by default if no value is given
	#[serde(default = "prefix_default")]
	pub prefix: String,
	/// The discord user IDs of the bot owners/admins
	#[serde(default)]
	pub owners: HashSet<UserId>,
	/// Pre defined classes, these are loaded into the whitelist on startup.
	#[serde(default)]
	pub class_whitelist: HashSet<String>,
}

fn prefix_default() -> String {
	"~".to_owned()
}

impl Default for General {
	fn default() -> Self {
		Self {
			discord_token: "".to_owned(),
			prefix: "~".to_owned(),
			owners: HashSet::new(),
			class_whitelist: HashSet::new(),
		}
	}
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