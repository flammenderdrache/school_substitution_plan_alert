#![allow(clippy::non_ascii_literal)]
#![allow(clippy::let_underscore_drop)]

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use chrono::{Datelike, DateTime, Local};
use log::{debug, error, info, LevelFilter, trace};
use simple_logger::SimpleLogger;
use uuid::Uuid;

use crate::config::Config;
use crate::discord::DiscordNotifier;
use crate::substitution_pdf_getter::{SubstitutionPDFGetter, Weekdays};
use crate::substitution_schedule::SubstitutionSchedule;

mod substitution_schedule;
mod tabula_json_parser;
mod substitution_pdf_getter;
mod discord;
mod config;

const PDF_JSON_ROOT_DIR: &str = "./pdf-jsons";
const TEMP_ROOT_DIR: &str = "/tmp/school-substitution-scanner-temp-dir";
const USER_AND_CLASSES_SAVE_LOCATION: &str = "./class_registry.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	SimpleLogger::new()
		.with_level(LevelFilter::Error)
		.with_module_level("school_substitution_plan_alert", LevelFilter::Debug)
		.init()
		.unwrap();

	// Make sure the paths we want to use exist
	std::fs::create_dir_all(TEMP_ROOT_DIR)?;
	std::fs::create_dir_all(PDF_JSON_ROOT_DIR)?;

	let config_file = std::fs::File::open("./config.toml").expect("Error opening config file");
	let config = Config::from_file(config_file);

	let discord_notifier = Arc::from(discord::DiscordNotifier::new(config).await);

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
		tokio::spawn(async move {
			if let Err(why) = check_weekday_pdf(next_valid_school_weekday, pdf_getter_arc, discord_notifier_arc).await {
				error!("{}", why);
			}
		});

		let pdf_getter_arc = pdf_getter.clone();
		let discord_notifier_arc = discord_notifier.clone();
		tokio::spawn(async move {
			if let Err(why) = check_weekday_pdf(day_after, pdf_getter_arc, discord_notifier_arc).await {
				error!("{}", why);
			}
		});

		counter += 1;
		debug!("Loop ran {} times", counter);
		trace!("Loop end before sleep");
		tokio::time::sleep(Duration::from_secs(20)).await;
	}
}

async fn check_weekday_pdf(day: Weekdays, pdf_getter: Arc<SubstitutionPDFGetter<'_>>, discord: Arc<DiscordNotifier>) -> Result<(), Box<dyn std::error::Error>> {
	info!("Checking PDF for {}", day);
	let temp_dir_path = make_temp_dir();
	let temp_file_name = get_random_name();
	let temp_file_path = format!("{}/{}", temp_dir_path, temp_file_name);
	let temp_file_path = Path::new(&temp_file_path);

	let pdf = pdf_getter.get_weekday_pdf(day).await?;
	let mut temp_pdf_file = std::fs::File::create(temp_file_path).expect("Couldn't create temp pdf file");
	temp_pdf_file.write_all(&pdf)?;
	let new_schedule = SubstitutionSchedule::from_pdf(temp_file_path)?;
	let classes = discord.get_classes().await;

	//This is only still here while testing the new loop. Will be removed at the next version
	// for class in classes {
	// 	if let Some(new_substitutions) = new_schedule.get_substitutions(class.as_str()) {
	// 		if let Ok(old_schedule_json) = std::fs::File::open(format!("./{}/{}.json", PDF_JSON_ROOT_DIR, day)) {
	// 			let old_schedule: SubstitutionSchedule = serde_json::from_reader(old_schedule_json).expect("For some reason the json of the old PDF was malformed.");
	// 			if let Some(old_substitutions) = old_schedule.get_substitutions(class.as_str()) {
	// 				if new_substitutions != old_substitutions {
	// 					discord.notify_users_for_class(class.as_str(), day).await?;
	// 				}
	// 			}
	// 		} else {
	// 			discord.notify_users_for_class(class.as_str(), day).await?;
	// 		}
	// 	}
	// }

	//Open and parse the json file first, instead of at each iteration in the loop
	let old_schedule_option: Option<SubstitutionSchedule> = {
		let old_json_file = std::fs::OpenOptions::new()
			.read(true)
			.write(false)
			.open(format!("./{}/{}.json", PDF_JSON_ROOT_DIR, day));

		if let Ok(old_schedule_json) = old_json_file {
			match serde_json::from_reader(old_schedule_json) {
				Ok(old_schedule) => { Some(old_schedule) }
				Err(why) => {
					error!("{}", why);
					panic!("Error parsing the old json");
				}
			}
		} else {
			None
		}
	};

	if let Some(old_schedule) = old_schedule_option {
		if !new_schedule.get_entries().keys().all(|k| old_schedule.get_entries().contains_key(k)) {
			discord.notify_users_for_classes(day, &new_schedule).await?;
		}
	} else {
		discord.notify_users_for_classes(day, &new_schedule).await?;
	}

	let new_substitution_json = serde_json::to_string_pretty(&new_schedule).expect("Couldn't write the new Json");
	let mut substitution_file = OpenOptions::new()
		.write(true)
		.create(true)
		.truncate(true)
		.open(format!("{}/{}.json", PDF_JSON_ROOT_DIR, day))
		.expect("Couldn't open file to write new json");

	substitution_file.write_all(new_substitution_json.as_bytes())?;

	std::fs::remove_file(temp_file_path)?;
	std::fs::remove_dir(temp_dir_path)?;
	Ok(())
}

fn get_random_name() -> String {
	trace!("Returning random name");
	format!("{}", Uuid::new_v4())
}

fn make_temp_dir() -> String {
	trace!("Creating temp directory");
	let temp_dir_name = get_random_name();
	let temp_dir = format!("{}/{}", TEMP_ROOT_DIR, temp_dir_name);
	std::fs::create_dir(Path::new(&temp_dir)).expect("Could not create temp dir");
	temp_dir
}