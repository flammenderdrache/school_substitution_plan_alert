use std::fs::File;

use crate::substitution_schedule::SubstitutionSchedule;

mod substitution_schedule;
mod tabula_json_parser;

fn main() {
	let mut text_as_vec = match tabula_json_parser::parse(File::open("tabula/1337.json").unwrap()) {
		Ok(str) => { str }
		Err(why) => { panic!("{}", why) }
	};

	let substitutions = SubstitutionSchedule::from_csv(&mut text_as_vec);
	println!("{}", substitutions);
}