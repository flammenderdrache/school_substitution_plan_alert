use std::collections::HashSet;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Mutex;

use crate::substitution_pdf_getter::Weekdays;

const PDF_JSON_DIR_NAME: &str = "pdf_jsons";
const WHITELIST_JSON_FILE_NAME: &str = "class_whitelist.json";

pub struct Data {
	data_directory: String,
	pdf_json_dir: String,
	whitelist_file: Mutex<File>,
}

impl Data {
	pub fn default(data_directory: String) -> Result<Self, Box<dyn std::error::Error>> {
		std::fs::create_dir_all(data_directory.as_str())?;
		std::fs::create_dir_all(format!("{}/{}", data_directory, PDF_JSON_DIR_NAME))?;

		let whitelist_file = std::fs::OpenOptions::new()
			.read(true)
			.write(true)
			.create(true)
			.open(format!("{}/{}", data_directory, WHITELIST_JSON_FILE_NAME))?;

		Ok(Self {
			data_directory,
			pdf_json_dir: "pdf_jsons".to_owned(),
			whitelist_file: Mutex::new(whitelist_file),
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

	fn update_class_whitelist(&self, classes: &HashSet<String>) -> Result<(), Box<dyn Error + '_>> {
		let mut class_whitelist_file = self.whitelist_file.lock()?;
		class_whitelist_file.seek(SeekFrom::Start(0))?; //Make sure the File Read/Write cursor is at the beginning of the file before reading
		let mut class_whitelist: HashSet<String> = serde_json::from_reader(&*class_whitelist_file).unwrap_or_default();

		let mut changed = false;
		for class in classes {
			if !class_whitelist.contains(class) {
				class_whitelist.insert(class.clone());
				changed = true;
			}
		}


		if changed {
			let whitelist_json = serde_json::to_string_pretty(&class_whitelist).unwrap();
			class_whitelist_file.set_len(0)?;
			class_whitelist_file.seek(SeekFrom::Start(0))?;
			class_whitelist_file.write_all(whitelist_json.as_bytes())?;
		}

		Ok(())
	}

	fn get_class_whitelist(&self) -> Result<HashSet<String>, Box<dyn Error + '_>> {
		let mut class_whitelist_file = self.whitelist_file.lock()?;
		class_whitelist_file.seek(SeekFrom::Start(0));
		let class_whitelist: HashSet<String> = serde_json::from_reader(&*class_whitelist_file)?;
		Ok(class_whitelist)
	}
}


pub trait DataStore {
	/// Stores the pdf json
	fn store_pdf_json(&self, weekday: Weekdays, pdf_json: &str) -> Result<(), Box<dyn std::error::Error>>;

	/// Retrieves a pdf json from the datastore
	fn get_pdf_json(&self, weekday: Weekdays) -> Result<String, Box<dyn std::error::Error>>;

	/// Stores the class whitelist or updates it with new data
	fn update_class_whitelist(&self, classes: &HashSet<String>) -> Result<(), Box<dyn std::error::Error + '_>>;

	/// Retrieves the class whitelist from the datastore
	fn get_class_whitelist(&self) -> Result<HashSet<String>, Box<dyn Error + '_>>;
}

#[cfg(test)]
mod tests {
	use crate::util::get_random_name;

	use super::*;

	#[test]
	fn test_store_and_retrieve_pdf_json() {
		let data_directory = format!("/tmp/test-{}", get_random_name());
		println!("tmp directory: {}", data_directory);
		let data = Data::default(data_directory).unwrap();


		let json = "{ test: \"this is a test\" }".to_owned();

		data.store_pdf_json(Weekdays::Monday, json.as_str()).unwrap();

		assert_eq!(json, data.get_pdf_json(Weekdays::Monday).unwrap())
	}

	#[test]
	fn test_update_and_get_whitelist_json() {
		let data_directory = format!("/tmp/test-{}", get_random_name());
		println!("tmp directory: {}", data_directory);
		let data = Data::default(data_directory).unwrap();

		let mut first_classes = HashSet::new();
		first_classes.insert("TEST1".to_owned());
		first_classes.insert("TEST2".to_owned());

		data.update_class_whitelist(&first_classes).unwrap();

		assert_eq!(first_classes, data.get_class_whitelist().unwrap(),
				   "Returned whitelist does not equal the whitelist given to store"
		);

		let mut second_classes = HashSet::new();
		second_classes.insert("TEST3".to_owned());
		second_classes.insert("TEST4".to_owned());

		data.update_class_whitelist(&second_classes).unwrap();

		first_classes.extend(second_classes);
		let both = first_classes;

		assert_eq!(both, data.get_class_whitelist().unwrap())
	}
}