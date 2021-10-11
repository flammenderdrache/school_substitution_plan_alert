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
