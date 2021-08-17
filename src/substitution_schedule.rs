use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use lopdf::Document;
use chrono::{Local, NaiveDate, Utc, Offset, Date};
use std::fmt::{Display, Formatter};
use std::clone::Clone;
use std::io::Read;
use crate::tabula_json_parser::parse;
use std::path::Path;
use std::fs::File;
use std::str;
use std::process::Command;
use std::ffi::OsStr;
use std::time::SystemTime;

#[derive(Serialize, Deserialize, PartialOrd, PartialEq)]
pub struct Substitutions {
	#[serde(rename(serialize = "0"))]
	#[serde(skip_serializing_if = "String::is_empty")]
	pub block_0: String,
	#[serde(rename(serialize = "1"))]
	#[serde(skip_serializing_if = "String::is_empty")]
	pub block_1: String,
	#[serde(rename(serialize = "2"))]
	#[serde(skip_serializing_if = "String::is_empty")]
	pub block_2: String,
	#[serde(rename(serialize = "3"))]
	#[serde(skip_serializing_if = "String::is_empty")]
	pub block_3: String,
	#[serde(rename(serialize = "4"))]
	#[serde(skip_serializing_if = "String::is_empty")]
	pub block_4: String,
	#[serde(rename(serialize = "5"))]
	#[serde(skip_serializing_if = "String::is_empty")]
	pub block_5: String,
}

impl Substitutions {
	pub fn new() -> Self {
		Self {
			block_0: "".to_string(),
			block_1: "".to_string(),
			block_2: "".to_string(),
			block_3: "".to_string(),
			block_4: "".to_string(),
			block_5: "".to_string()
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct SubstitutionSchedule {
	/// The creation date inside the PDF
	pdf_create_date: i64,
	/// The name of the class is the Key and the Value is a Substitutions struct
	entries: HashMap<String, Substitutions>,
	/// The time when the struct was created, used for comparing the age
	struct_time: u64,
}

impl SubstitutionSchedule {
	pub fn from_table(table: &Vec<Vec<String>>, pdf_create_date: i64) -> Self {
		let mut entries: HashMap<String, Substitutions> = HashMap::new();

		let classes = &table[0][1..];

		for class in classes {
			entries.insert(class.to_string(), Substitutions::new());
		}

		let mut row = 1;

		for lesson_idx in 0..5 {
			loop {
				for (i, substitution_part) in table[row][1..].iter().enumerate() {

					let substitutions = entries.get_mut(&classes[i]).unwrap();

					let mut block = match lesson_idx {
						0 => &mut substitutions.block_0,
						1 => &mut substitutions.block_1,
						2 => &mut substitutions.block_2,
						3 => &mut substitutions.block_3,
						4 => &mut substitutions.block_4,
						5 => &mut substitutions.block_5,
						_ => panic!("more then 5 lessons used"),
					};

					if !substitution_part.is_empty() {
						if block.is_empty() {
							block.push_str(substitution_part);
						} else {
							block.push_str(&format!("\n{}", substitution_part.to_owned()));
						}
					}
				}

				if table[row][0].starts_with("-") {
					break
				} else {
					row += 1;
				}
			}

			row += 1;
		}

		let now = SystemTime::now();
		let since_the_epoch = now
			.duration_since(SystemTime::UNIX_EPOCH)
			.expect("Time got fucked");
		let time_millis = since_the_epoch.as_millis() as u64;

		Self {
			pdf_create_date,
			entries,
			struct_time: time_millis,
		}
	}

	pub fn from_pdf<T: AsRef<Path> + AsRef<OsStr>>(path: T) -> Result<Self, Box<dyn std::error::Error>> {
		let pdf = Document::load(&path).unwrap().extract_text(&[1]).unwrap();

		let date_idx_start = pdf.find("Datum: ").ok_or("date not found")?;
		let date_idx_end = pdf[date_idx_start..].find("\n").ok_or("date end not found")? + date_idx_start;

		let date_str: Vec<u32> = pdf[date_idx_start..date_idx_end].split(", ")
			.last()
			.ok_or("date string has no ','")?
			.split(".")
			.collect::<Vec<&str>>()
			.iter()
			.map(|s| (*s).parse::<u32>().unwrap())
			.collect();

		let date = chrono::Date::<Local>::from_utc(
			NaiveDate::from_ymd(date_str[2] as i32, date_str[1], date_str[0]),
			Utc.fix()
		).and_hms(0, 0, 0).timestamp();

		let output = Command::new("java")
			.arg("-jar")
			.arg("./tabula/tabula.jar")
			.arg("-g")
			.arg("-f")
			.arg("JSON")
			.arg(path)
			.output()?;

		let table = parse(str::from_utf8(&output.stdout).unwrap())?;

		Ok(Self::from_table(&table, date))
	}

	pub fn get_substitutions(&self, class: &str) -> Option<&Substitutions> {
		self.entries.get(class)
	}

	pub fn get_date(&self) -> i64 {
		self.pdf_create_date
	}
}

impl Display for SubstitutionSchedule {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
	}
}