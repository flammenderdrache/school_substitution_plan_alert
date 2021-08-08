mod jonas;
mod tabula_json_parser;

use std::fs::File;
use serde_json::Value;
use std::fmt::Formatter;
use std::fmt::Display;
use serde::{Serialize, Deserialize};
use crate::jonas::SubstitutionSchedule;
use std::error::Error;

fn main() {
	let mut text_as_vec = match tabula_json_parser::parse_file(std::fs::File::open("tabula/1337.json").unwrap()) {
		Ok(str) => {str}
		Err(why) => {panic!("{}", why)}
	};

	println!("{}", json);
	
	let substitutions = SubstitutionSchedule::from_csv(&mut text_as_vec);
	println!("{}", substitutions);
}