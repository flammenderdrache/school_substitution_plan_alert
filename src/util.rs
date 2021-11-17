use std::path::Path;

use log::trace;
use uuid::Uuid;

use crate::TEMP_ROOT_DIR;

pub fn get_random_name() -> String {
	trace!("Returning random name");
	format!("{}", Uuid::new_v4())
}

pub fn make_temp_dir() -> String {
	trace!("Creating temp directory");
	let temp_dir_name = get_random_name();
	let temp_dir = format!("{}/{}", TEMP_ROOT_DIR, temp_dir_name);
	std::fs::create_dir(Path::new(&temp_dir)).expect("Could not create temp dir");
	temp_dir
}

/// Removes the dots to make e.g. "BGYM19.1" valid (turning it into "BGYM191")
/// Also turns the input uppercase; "BGym19.1" -> "BGYM191" as that is how they are referred to in the PDF
pub fn sanitize_and_check_register_class_input(input: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
	let input = input.replace('.', "");

	if input.len() < 4 {
		return Err("Argument too short".into());
	}

	if !(input.contains(char::is_alphabetic) &&
		input.contains(|c: char| c.is_ascii_digit())) {
		return Err("Argument is incorrectly formatted".into());
	}

	let input = input.to_uppercase();

	Ok(input)
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_sanitize_should_pass() {
		let test_class = "BGYM191";
		let output = sanitize_and_check_register_class_input(test_class).unwrap();
		assert_eq!(output, test_class);
	}

	#[test]
	#[should_panic]
	fn test_sanitize_too_short() {
		let test_class = "B2";
		let _ = sanitize_and_check_register_class_input(test_class).unwrap();
	}

	#[test]
	fn test_sanitize_remove_dots() {
		let test_class = "BGYM19.1";
		let output = sanitize_and_check_register_class_input(test_class).unwrap();
		assert_eq!(output, "BGYM191");
	}

	#[test]
	#[should_panic]
	fn test_sanitize_missing_class_number() {
		let test_class = "ELIAS";
		let _ = sanitize_and_check_register_class_input(test_class).unwrap();
	}

	#[test]
	#[should_panic]
	fn test_sanitize_only_numbers() {
		let test_class = "1234567420";
		let _ = sanitize_and_check_register_class_input(test_class).unwrap();
	}

	#[test]
	#[should_panic]
	fn test_sanitize_check_between_large_char_and_small_char_ascii_value() {
		let test_class = "BGY/@;19[1";
		let output = sanitize_and_check_register_class_input(test_class).unwrap();
		assert_eq!(output, "BGYM191");
	}
}