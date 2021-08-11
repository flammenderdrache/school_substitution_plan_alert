use crate::substitution_schedule::SubstitutionSchedule;

mod substitution_schedule;
mod tabula_json_parser;

fn main() -> Result<(), Box<dyn std::error::Error>>{
	let substitutions = SubstitutionSchedule::from_pdf("tabula/42069");
	println!("{}", substitutions?);

	Ok(())
}