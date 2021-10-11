use std::fs::OpenOptions;
use std::io::Write;
use crate::substitution_pdf_getter::Weekdays;

pub struct Data {
	data_directory: String
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
	fn get_pdf_json(&self, weekday: Weekdays) {
		todo!()
	}
}

pub trait DataStore {
	/// Stores the pdf json
	fn store_pdf_json(&self, weekday: Weekdays, pdf_json: String) -> Result<(), Box<dyn std::error::Error>>;

	/// Retrieves a pdf json from the datastore
	fn get_pdf_json(&self, weekday: Weekdays);
}