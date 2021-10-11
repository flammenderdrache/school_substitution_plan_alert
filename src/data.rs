use std::fs::OpenOptions;
use std::io::{Read, Write};

use crate::substitution_pdf_getter::Weekdays;

pub struct Data {
	data_directory: String,
	pdf_json_dir: String,
}

impl Data {
	pub fn default(data_directory: String) -> Self {
		Self {
			data_directory,
			pdf_json_dir: "pdf_jsons".to_owned(),
		}
	}
}

impl DataStore for Data {
	/// Stores the given PDF Json in a file
	fn store_pdf_json(&self, weekday: Weekdays, pdf_json: String) -> Result<(), Box<dyn std::error::Error>> {
		let mut substitution_file = OpenOptions::new()
			.write(true)
			.create(true)
			.truncate(true)
			.open(format!("{}/{}/{}.json", self.data_directory, "pdf_jsons", weekday))
			.expect("Couldn't open file to write new json");

		substitution_file.write_all(pdf_json.as_bytes())?;

		Ok(())
	}

	/// Retrieves the pdf Json from a file
	fn get_pdf_json(&self, weekday: Weekdays) -> Result<String, Box<dyn std::error::Error>> {
		let mut old_json_file = std::fs::OpenOptions::new()
			.read(true)
			.write(false)
			.open(format!("./{}/{}/{}.json", self.data_directory, "pdf_jsons", weekday))?;

		let mut content = String::new();

		old_json_file.read_to_string(&mut content)?;

		Ok(content)
	}
}

pub trait DataStore {
	/// Stores the pdf json
	fn store_pdf_json(&self, weekday: Weekdays, pdf_json: String) -> Result<(), Box<dyn std::error::Error>>;

	/// Retrieves a pdf json from the datastore
	fn get_pdf_json(&self, weekday: Weekdays) -> Result<String, Box<dyn std::error::Error>>;
}

mod tests {
	use crate::util::get_random_name;
	use super::*;

	#[test]
	fn test_store_and_retrieve_json() {
		let data_directory = format!("/tmp/test-{}", get_random_name());

		std::fs::create_dir_all(data_directory.clone());

		let data = Data::default(data_directory);

		let json = "{ test: \"this is a test\"".to_owned();

		data.store_pdf_json(Weekdays::Monday, json.clone()).unwrap();

		assert_eq!(json, data.get_pdf_json(Weekdays::Monday).unwrap())
	}
}