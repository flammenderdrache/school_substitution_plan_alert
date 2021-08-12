use crate::substitution_schedule::SubstitutionSchedule;
use crate::substitution_pdf_getter::*;
use reqwest::Client;
use std::time::Duration;
use std::io::Write;

mod substitution_schedule;
mod tabula_json_parser;
mod substitution_pdf_getter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// let schedule1 = SubstitutionSchedule::from_pdf("./tabula/1337").unwrap();
	// let schedule2 = SubstitutionSchedule::from_pdf("./tabula/42069").unwrap();
	//
	// let sub1 = schedule1.get_substitutions("BGYM191").unwrap();
	// let sub2 = schedule2.get_substitutions("BGYM191").unwrap();
	//
	// println!("{}\n\n{}", schedule1, schedule2);
	//
	// println!("{}", sub1 == sub2);


	// let client = Client::builder()
	// 	.connect_timeout(Duration::from_secs(20))
	// 	.timeout(Duration::from_secs(20))
	// 	.build()?;
	//
	// let test_request = client
	// 	.get("https://buessing.schule/plaene/VertretungsplanA4_Donnerstag.pdf")
	// 	.header("Authorization", "Basic aGJzdXNlcjpoYnNwYXNz")
	// 	.build()?;
	//
	// let response = client.execute(test_request).await?;
	// let mut bytes = response.bytes().await?;


	let pdf_getter = SubstitutionPDFGetter::default();
	let pdf_data = pdf_getter.get_weekday_pdf(Weekdays::Wednesday).await?;

	let mut new_pdf = std::fs::File::create("./download/Mittwoch.pdf")?;

	new_pdf.write_all(pdf_data.as_ref())?;

	Ok(())
}