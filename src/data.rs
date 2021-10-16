use std::error::Error;
use std::fs::OpenOptions;
use std::io::{Read, Write};

use crate::substitution_pdf_getter::Weekdays;

const PDF_JSON_DIR_NAME: &str = "pdf_jsons";

pub struct Data {
	data_directory: String,
	pdf_json_dir: String,
}

impl Data {
	pub fn default(data_directory: String) -> Result<Self, Box<dyn std::error::Error>> {
		std::fs::create_dir_all(data_directory.as_str())?;
		std::fs::create_dir_all(format!("{}/{}", data_directory, PDF_JSON_DIR_NAME))?;

		Ok(Self {
			data_directory,
			pdf_json_dir: "pdf_jsons".to_owned(),
		})
	}
}

impl DataStore for Data {
	/// Stores the given PDF Json in a file
	fn store_pdf_json(&self, weekday: Weekdays, pdf_json: &str) -> Result<(), Box<dyn std::error::Error>> {
		let mut substitution_file = OpenOptions::new()
			.write(true)
			.create(true)
			.truncate(true)
			.open(format!("{}/{}/{}.json", self.data_directory, PDF_JSON_DIR_NAME, weekday))
			.expect("Couldn't open file to write new json");

		substitution_file.write_all(pdf_json.as_bytes())?;

		Ok(())
	}

	/// Retrieves the pdf Json from a file
	fn get_pdf_json(&self, weekday: Weekdays) -> Result<String, Box<dyn std::error::Error>> {
		let mut old_json_file = std::fs::OpenOptions::new()
			.read(true)
			.write(false)
			.open(format!("{}/{}/{}.json", self.data_directory, PDF_JSON_DIR_NAME, weekday))?;

		let mut content = String::new();

		old_json_file.read_to_string(&mut content)?;

		Ok(content)
	}

	fn store_class_whitelist(&self, class_whitelist: &str) -> Result<(), Box<dyn Error>> {
		todo!()
	}

	fn get_class_whitelist(&self) -> Result<String, Box<dyn Error>> {
		todo!()
	}
}

pub trait DataStore {
	/// Stores the pdf json
	fn store_pdf_json(&self, weekday: Weekdays, pdf_json: &str) -> Result<(), Box<dyn std::error::Error>>;

	/// Retrieves a pdf json from the datastore
	fn get_pdf_json(&self, weekday: Weekdays) -> Result<String, Box<dyn std::error::Error>>;

	/// Stores the class whitelist
	fn store_class_whitelist(&self, class_whitelist: &str) -> Result<(), Box<dyn std::error::Error>>;

	/// Retrieves the class whitelist from the datastore
	fn get_class_whitelist(&self) -> Result<String, Box<dyn std::error::Error>>;
}

mod tests {
	use crate::util::get_random_name;

	use super::*;

	#[test]
	fn test_store_and_retrieve_json() {
		let data_directory = format!("/tmp/test-{}", get_random_name());

		std::fs::create_dir_all(data_directory.clone()).unwrap();

		let data = Data::default(data_directory).unwrap();

		let json = "{ test: \"this is a test\" }".to_owned();

		data.store_pdf_json(Weekdays::Monday, json.as_str()).unwrap();

		assert_eq!(json, data.get_pdf_json(Weekdays::Monday).unwrap())
	}
}