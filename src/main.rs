#![allow(clippy::non_ascii_literal)]
#![allow(clippy::let_underscore_drop)]

use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use chrono::{Datelike, DateTime, Local};
use log::{debug, error, info, LevelFilter, trace};
use serenity::prelude::TypeMapKey;
use simple_logger::SimpleLogger;
use tokio::sync::Mutex;

use crate::config::Config;
use crate::data::{Data, DataStore};
use crate::discord::{ClassesAndUsers, DiscordNotifier};
use crate::substitution_pdf_getter::{SubstitutionPDFGetter, Weekdays};
use crate::substitution_schedule::SubstitutionSchedule;

mod substitution_schedule;
mod tabula_json_parser;
mod substitution_pdf_getter;
mod discord;
mod config;
mod data;
mod util;
mod error;

const TEMP_ROOT_DIR: &str = "/tmp/school-substitution-scanner-temp-dir";
const USER_AND_CLASSES_SAVE_LOCATION: &str = "./class_registry.json";
const CLASS_WHITELIST_LOCATION: &str = "./class_whitelist.json";
static SOURCE_URLS: [&str; 5] = [
	"https://buessing.schule/plaene/VertretungsplanA4_Montag.pdf",
	"https://buessing.schule/plaene/VertretungsplanA4_Dienstag.pdf",
	"https://buessing.schule/plaene/VertretungsplanA4_Mittwoch.pdf",
	"https://buessing.schule/plaene/VertretungsplanA4_Donnerstag.pdf",
	"https://buessing.schule/plaene/VertretungsplanA4_Freitag.pdf",
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	SimpleLogger::new()
		.with_level(LevelFilter::Error)
		.with_module_level("school_substitution_plan_alert", LevelFilter::Debug)
		.init()
		.unwrap();

	// Make sure the paths we want to use exist
	std::fs::create_dir_all(TEMP_ROOT_DIR)?;

	let config_file = std::fs::File::open("./config.toml").expect("Error opening config file");
	let config = Config::from_file(config_file);
	let datastore = Arc::new(Data::new("./data".to_owned())?);

	let whitelist_config_file = std::fs::OpenOptions::new()
		.read(true)
		.write(true)
		.create(true)
		.open(CLASS_WHITELIST_LOCATION)
		.expect("Couldn't open whitelist config file");

	// update_whitelisted_classes(&config.general.class_whitelist, &mut whitelist_config_file)?;

	if let Err(why) = datastore.update_class_whitelist(&config.general.class_whitelist) {
		log::error!("{}", why)
	}

	let discord_notifier = Arc::from(discord::DiscordNotifier::new(config).await);

	{
		let file_mutex = Arc::from(Mutex::new(whitelist_config_file));

		let mut data = discord_notifier.data.write().await;
		data.insert::<WhitelistFile>(file_mutex);
	}

	let pdf_getter = Arc::new(SubstitutionPDFGetter::default());

	let mut counter: u32 = 0;
	info!("Starting loop");
	loop {
		trace!("Loop start");

		let local: DateTime<Local> = Local::now();
		let next_valid_school_weekday = Weekdays::from(local.weekday());
		let day_after = next_valid_school_weekday.next_day();

		debug!("Local day: {}; next valid school day: {}; day after that: {}", local.weekday(), next_valid_school_weekday, day_after);


		let pdf_getter_arc = pdf_getter.clone();
		let discord_notifier_arc = discord_notifier.clone();
		let datastore_arc = datastore.clone();
		tokio::spawn(async move {
			if let Err(why) = check_weekday_pdf(next_valid_school_weekday, pdf_getter_arc, discord_notifier_arc, datastore_arc).await {
				error!("{}", why);
			}
		});

		let pdf_getter_arc = pdf_getter.clone();
		let discord_notifier_arc = discord_notifier.clone();
		let datastore_arc = datastore.clone();
		tokio::spawn(async move {
			if let Err(why) = check_weekday_pdf(day_after, pdf_getter_arc, discord_notifier_arc, datastore_arc).await {
				error!("{}", why);
			}
		});

		counter += 1;
		debug!("Loop ran {} times", counter);
		trace!("Loop end before sleep");
		tokio::time::sleep(Duration::from_secs(20)).await;
	}
}

#[allow(clippy::or_fun_call)]
async fn check_weekday_pdf(day: Weekdays, pdf_getter: Arc<SubstitutionPDFGetter<'_>>, discord: Arc<DiscordNotifier>, datastore: Arc<Data>) -> Result<(), Box<dyn std::error::Error>> {
	info!("Checking PDF for {}", day);
	let temp_dir_path = util::make_temp_dir();
	let temp_file_name = util::get_random_name();
	let temp_file_path = format!("{}/{}", temp_dir_path, temp_file_name);
	let temp_file_path = Path::new(&temp_file_path);

	let pdf = pdf_getter.get_weekday_pdf(day).await?;
	let mut temp_pdf_file = std::fs::File::create(temp_file_path).expect("Couldn't create temp pdf file");
	temp_pdf_file.write_all(&pdf)?;
	let new_schedule = SubstitutionSchedule::from_pdf(temp_file_path)?;

	// Check the date in the pdf and if it is too old delete the file (if it exists) and return.
	if new_schedule.pdf_create_date < chrono::Local::today().and_hms_milli(0, 0, 0, 0).timestamp_millis() {
		log::info!("Deleting old pdf for day {}", &day);
		datastore.delete_pdf_json(day)?;
		return Ok(());
	}

	if let Err(why) = datastore.update_class_whitelist(&new_schedule.get_classes()) {
		log::error!("{}", why);
	}

	let old_schedule_option: Option<SubstitutionSchedule> = {
		match datastore.get_pdf_json(day) {
			Ok(content) => {
				log::trace!("old_schedule_option datastore pdf was Ok");
				match serde_json::from_str(content.as_str()) {
					Ok(old_schedule) => Some(old_schedule),
					Err(why) => {
						log::error!("{}", why);
						None
					}
				}
			}
			Err(err) => {
				log::error!("{}", err);
				None
			}
		}
	};

	let data = discord.data.read().await;

	let classes_and_users = data.get::<ClassesAndUsers>().unwrap();
	let classes_and_users_inner = classes_and_users.get_inner_classes_and_users();

	let mut to_notify: HashSet<u64> = HashSet::new();

	let mut add_to_notify = |class| {
		for user_id in classes_and_users_inner.get(class).unwrap() { // The unwrap is safe since we know the class exists
			to_notify.insert(*user_id);
		}
	};

	for class in classes_and_users_inner.keys() {
		if let Some(new_substitutions) = new_schedule.get_substitutions(class.as_str()) {
			if let Some(old_schedule) = &old_schedule_option {
				if let Some(old_substitutions) = old_schedule.get_substitutions(class.as_str()) {
					if new_substitutions != old_substitutions {
						add_to_notify(class);
					}
				}
			} else {
				add_to_notify(class);
			}
		}
	}

	discord.notify_users(day, &new_schedule, to_notify).await?;

	let new_schedule_json = serde_json::to_string_pretty(&new_schedule).expect("Couldn't write the new Json");

	datastore.store_pdf_json(day, new_schedule_json.as_str())?;

	std::fs::remove_file(temp_file_path)?;
	std::fs::remove_dir(temp_dir_path)?;

	Ok(())
}

struct WhitelistFile {}

impl TypeMapKey for WhitelistFile {
	type Value = Arc<Mutex<File>>;
}