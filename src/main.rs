use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use chrono::{Datelike, DateTime, Local};
use uuid::Uuid;

use crate::substitution_pdf_getter::{SubstitutionPDFGetter, Weekdays};
use crate::substitution_schedule::{SubstitutionSchedule, Substitutions};
use simple_logger::SimpleLogger;
use log::LevelFilter;

mod substitution_schedule;
mod tabula_json_parser;
mod substitution_pdf_getter;
mod discord;

const PDF_JSON_ROOT_DIR: &str = "./pdf-jsons";
const TEMP_ROOT_DIR: &str = "/tmp/school-substitution-scanner-temp-dir";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	SimpleLogger::new()
		.with_level(LevelFilter::Error)
		.with_module_level("school_substitution_plan_alert", LevelFilter::Debug)
		.init()
		.unwrap();

	//Load the environment variables from the .env file
	dotenv::dotenv().expect("Failed to load env file");

	// Make sure the paths we want to use exist
	std::fs::create_dir_all(TEMP_ROOT_DIR)?;
	std::fs::create_dir_all(PDF_JSON_ROOT_DIR)?;

	let mut counter: u32 = 0;

	let pdf_getter = Arc::new(SubstitutionPDFGetter::default());

	let token = std::env::var("DISCORD_TOKEN").expect("Couldn't find the discord token in environment");
	let prefix = std::env::var("PREFIX").expect("Couldn't find the prefix in environment");
	let discord_notifier = discord::DiscordNotifier::new(token.as_str(), prefix.as_str()).await;

	log::info!("Starting loop");
	loop {
		log::trace!("Loop start");

		let local: DateTime<Local> = Local::now();
		let next_valid_school_weekday = Weekdays::from(local.weekday());
		let day_after = next_valid_school_weekday.next_day();

		log::debug!("Local day: {}; next valid school day: {}; day after that: {}", local.weekday(), next_valid_school_weekday, day_after);


		let pdf_getter_arc = pdf_getter.clone();
		tokio::spawn(async move {
			if let Err(why) = check_weekday_pdf(next_valid_school_weekday, pdf_getter_arc).await {
				log::error!("{}", why)
			}
		});

		let pdf_getter_arc = pdf_getter.clone();
		tokio::spawn(async move {
			if let Err(why) = check_weekday_pdf(day_after, pdf_getter_arc).await {
				log::error!("{}", why)
			}
		});

		counter += 1;
		log::debug!("Loop ran {} times", counter);
		log::trace!("Loop end before sleep");
		tokio::time::sleep(Duration::from_secs(20)).await;
	}
}

async fn check_weekday_pdf(day: Weekdays, pdf_getter: Arc<SubstitutionPDFGetter<'_>>) -> Result<(), Box<dyn std::error::Error>> {
	log::info!("Checking PDF for {}", day);
	let temp_dir_path = make_temp_dir();
	let temp_file_name = get_random_name();
	let temp_file_path = format!("{}/{}", temp_dir_path, temp_file_name);

	let temp_file_path = Path::new(&temp_file_path);

	let pdf = pdf_getter.get_weekday_pdf(day).await?;
	let mut temp_pdf_file = std::fs::File::create(temp_file_path).expect("Couldn't create temp pdf file");
	temp_pdf_file.write_all(&pdf)?;
	let new_schedule = SubstitutionSchedule::from_pdf(temp_file_path)?;
	let class = "BGYM191";

	if let Some(new_substitutions) = new_schedule.get_substitutions("BGYM191") {
		if let Ok(old_schedule_json) = std::fs::File::open(format!("./{}/{}.json", PDF_JSON_ROOT_DIR, day)) {
			let old_schedule: SubstitutionSchedule = serde_json::from_reader(old_schedule_json).expect("For some reason the json of the old PDF was malformed.");
			let old_substitutions = old_schedule.get_substitutions("BGYM191").unwrap(); //We save only when the class is there so unwrap is safe
			if new_substitutions != old_substitutions {
				notify_users(day, class, new_substitutions);
			}
		} else {
			notify_users(day, class, new_substitutions);
		}
	}

	let new_substitution_json = serde_json::to_string_pretty(&new_schedule).unwrap();
	let mut substitution_file = OpenOptions::new()
		.write(true)
		.create(true)
		.open(format!("{}/{}.json", PDF_JSON_ROOT_DIR, day))
		.expect("Couldn't open file to write new json");

	substitution_file.write_all(new_substitution_json.as_bytes())?;

	std::fs::remove_file(temp_file_path)?;
	std::fs::remove_dir(temp_dir_path)?;
	Ok(())
}

fn get_random_name() -> String {
	log::trace!("Returning random name");
	format!("{}", Uuid::new_v4())
}

fn make_temp_dir() -> String {
	log::trace!("Creating temp directory");
	let temp_dir_name = get_random_name();
	let temp_dir = format!("{}/{}", TEMP_ROOT_DIR, temp_dir_name);
	std::fs::create_dir(Path::new(&temp_dir)).expect("Could not create temp dir");
	temp_dir
}

#[allow(clippy::non_ascii_literal)]
fn notify_users(weekday: Weekdays, class: &str, substitutions: &Substitutions) {
	log::debug!("Change detected, notifying users for class {}", class);
	println!("Vertretungsplanänderung:\n Klasse {} am {} hat folgende Änderungen:\n {}", weekday, class, substitutions);
}