use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::process::Command;
use std::str;
use std::time::SystemTime;

use chrono::{Local, NaiveDate, Offset, Utc};
use lopdf::Document;
use serde::{Deserialize, Serialize};

use crate::tabula_json_parser::parse;

#[derive(Serialize, Deserialize, PartialOrd, PartialEq, Debug)]
pub struct Substitutions {
	#[serde(rename(serialize = "0"))]
	#[serde(rename(deserialize = "0"))]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub block_0: Option<String>,
	#[serde(rename(serialize = "1"))]
	#[serde(rename(deserialize = "1"))]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub block_1: Option<String>,
	#[serde(rename(serialize = "2"))]
	#[serde(rename(deserialize = "2"))]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub block_2: Option<String>,
	#[serde(rename(serialize = "3"))]
	#[serde(rename(deserialize = "3"))]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub block_3: Option<String>,
	#[serde(rename(serialize = "4"))]
	#[serde(rename(deserialize = "4"))]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub block_4: Option<String>,
	#[serde(rename(serialize = "5"))]
	#[serde(rename(deserialize = "5"))]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub block_5: Option<String>,
}

impl Substitutions {
	pub fn new() -> Self {
		Self {
			block_0: None,
			block_1: None,
			block_2: None,
			block_3: None,
			block_4: None,
			block_5: None,
		}
	}
	pub fn first_substitution(&self) -> usize {
		self.as_array().iter().position(|b| b.is_some()).unwrap_or(0)
	}

	pub fn last_substitution(&self) -> usize {
		self.as_array().iter().rposition(|b| b.is_some()).unwrap_or(5)
	}

	pub fn as_array(&self) -> [&Option<String>; 6] {
		// One could consider also implementing Iterator
		[&self.block_0, &self.block_1, &self.block_2, &self.block_3, &self.block_4, &self.block_5]
	}
}

impl Display for Substitutions {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubstitutionSchedule {
	/// The creation date inside the PDF
	pub pdf_create_date: i64,
	/// The name of the class is the Key and the Value is a Substitutions struct
	entries: HashMap<String, Substitutions>,
	/// The time when the struct was created, used for comparing the age
	struct_time: u64,
}

impl SubstitutionSchedule {
	#[allow(clippy::ptr_arg)]
	fn table_to_substitutions(table: &Vec<Vec<String>>) -> HashMap<String, Substitutions> {
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

					let block_option = match lesson_idx {
						0 => &mut substitutions.block_0,
						1 => &mut substitutions.block_1,
						2 => &mut substitutions.block_2,
						3 => &mut substitutions.block_3,
						4 => &mut substitutions.block_4,
						5 => &mut substitutions.block_5,
						_ => panic!("more then 5 lessons used"),
					};

					if !substitution_part.is_empty() {
						if let Some(block) = block_option {
							block.push_str(&format!("\n{}", substitution_part.clone()));
						} else {
							block_option.insert(substitution_part.clone());
						}
					}
				}

				if table[row][0].starts_with('-') {
					break;
				}
				row += 1;
			}

			row += 1;
		}

		entries
	}

	#[allow(clippy::ptr_arg)]
	pub fn from_table(tables: &Vec<Vec<Vec<String>>>, pdf_create_date: i64) -> Self {
		let mut entries = HashMap::new();

		for table in tables {
			entries.extend(Self::table_to_substitutions(table));
		}

		let time_now = SystemTime::now();
		let since_the_epoch = time_now
			.duration_since(SystemTime::UNIX_EPOCH)
			.expect("Time got fucked");

		#[allow(clippy::cast_possible_truncation)]
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
		let date_idx_end = pdf[date_idx_start..].find('\n').ok_or("date end not found")? + date_idx_start;

		let date_str: Vec<u32> = pdf[date_idx_start..date_idx_end].split(", ")
			.last()
			.ok_or("date string has no ','")?
			.split('.')
			.collect::<Vec<&str>>()
			.iter()
			.map(|s| (*s).parse::<u32>().unwrap())
			.collect();

		#[allow(clippy::cast_possible_wrap)]
		let date = chrono::Date::<Local>::from_utc(
			NaiveDate::from_ymd(date_str[2] as i32, date_str[1], date_str[0]),
			Utc.fix(),
		).and_hms_milli(0, 0, 0, 0).timestamp();

		let output = Command::new("java")
			.arg("-jar")
			.arg("./tabula/tabula.jar")
			.arg("-g")
			.arg("-f")
			.arg("JSON")
			.arg("-p")
			.arg("all")
			.arg(path)
			.output()?;

		let table = parse(str::from_utf8(&output.stdout).unwrap())?;

		Ok(Self::from_table(&table, date))
	}

	pub fn get_substitutions(&self, class: &str) -> Option<&Substitutions> {
		self.entries.get(class)
	}

	pub fn _get_entries(&self) -> &HashMap<String, Substitutions> { &self.entries }

	/// This function skips entries not present in the 'entries' `HashMap`
	#[allow(clippy::implicit_clone)]
	pub fn _get_entries_portion(&self, classes: &HashSet<&String>) -> HashMap<String, &Substitutions> {
		let mut portion = HashMap::new();

		for class in classes {
			if let Some(substitution) = self.entries.get(*class) {
				portion.insert(class.to_owned().to_owned(), substitution);
			}
		}

		portion
	}
}

impl Display for SubstitutionSchedule {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
	}
}