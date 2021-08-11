use crate::substitution_schedule::SubstitutionSchedule;

mod substitution_schedule;
mod tabula_json_parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let schedule1 = SubstitutionSchedule::from_pdf("./tabula/1337").unwrap();
	let schedule2 = SubstitutionSchedule::from_pdf("./tabula/42069").unwrap();

	let sub1 = schedule1.get_substitutions("BGYM191").unwrap();
	let sub2 = schedule2.get_substitutions("BGYM191").unwrap();

	println!("{}\n\n{}", schedule1, schedule2);

	println!("{}", sub1 == sub2);

	Ok(())
}