use std::collections::HashMap;
use serde::Serialize;
use chrono::{Local, NaiveDate, Utc, Offset, Date};
use std::fmt::{Display, Formatter};

#[derive(Serialize)]
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

#[derive(Serialize)]
pub struct SubstitutionSchedule {
	date: i64,
	entries: HashMap<String, Substitutions>,
}

impl SubstitutionSchedule {
	pub fn from_csv(csv: &mut Vec<Vec<String>>) -> Self {
		// let text_array: Vec<&str> = text.trim().split("\n").collect();
		//
		// fn extract_date(text_array: Vec<&str>) -> Date<Local> {
		//     let date_str: Vec<u32> = text_array[2].split(", ")
		//         .last()
		//         .unwrap()
		//         .split(".")
		//         .collect::<Vec<&str>>()
		//         .iter()
		//         .map(|s| (*s).parse::<u32>().unwrap())
		//         .collect();
		//
		//     chrono::Date::<Local>::from_utc(
		//         NaiveDate::from_ymd(date_str[2] as i32, date_str[1], date_str[0]),
		//         Utc.fix()
		//     )
		// }
		//
		// print!("{}", text_array[4]);
		//
		// let entries: HashMap<String, Substitutions> = HashMap::new();

		let mut entries: HashMap<String, Substitutions> = HashMap::new();

		let classes = &csv[0][1..];

		for class in classes {
			entries.insert(class.to_string(), Substitutions::new());
		}

		let mut line = 1;

		for lesson_idx in 0..5 {
			loop {
				for (i, substitution_part) in csv[line][1..].iter().enumerate() {

					let substitutions = entries.get_mut(&classes[i]).unwrap();

					let mut block = match lesson_idx {
						0 => &mut substitutions.block_0,
						1 => &mut substitutions.block_1,
						2 => &mut substitutions.block_2,
						3 => &mut substitutions.block_3,
						4 => &mut substitutions.block_4,
						5 => &mut substitutions.block_5,
						_ => panic!(""),
					};

					if block.is_empty() {
						block.push_str(substitution_part);
					} else {
						block.push_str(&format!("\n{}", substitution_part));
					}
				}

				if csv[line][0].starts_with("-") {
					break
				} else {
					line += 1;
				}
			}

			line += 1;
		}

		Self {
			date: 0,
			entries,
		}
	}
}

impl Display for SubstitutionSchedule {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
	}
}