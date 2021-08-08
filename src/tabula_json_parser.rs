use std::fs::File;
use serde_json::{Value, Error};
use std::fmt::Formatter;
use std::fmt::Display;
use serde::{Serialize, Deserialize};

pub fn parse(file: File) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
	let json: Value = serde_json::from_reader(file)?;
	let array = json.as_array().ok_or("Json malformed")?;
	let object = array[0].as_object().ok_or("Json malformed")?;
	let data = object.get("data").ok_or("Json data field missing")?;


	let mut rows = Vec::new();

	for row in data.as_array().ok_or("Json data missing")? {
		let row: Vec<Cell> = serde_json::from_value(row.clone())?;
		let row = Row {
			row
		};
		rows.push(row);
	}

	let mut rows_as_text = Vec::new();
	for mut row in rows {
		rows_as_text.push(row.extract_text());
	}

	Ok(rows_as_text)
}

#[derive(Debug, Deserialize, Serialize)]
struct Row {
	row: Vec<Cell>,
}

impl Row {
	pub fn extract_text(&mut self) -> Vec<String> {
		let mut text = Vec::new();
		for cell in &self.row {
			text.push(cell.text.clone())
		}

		text
	}
}

#[derive(Debug, Deserialize, Serialize)]
struct Cell {
	top: f64,
	left: f64,
	width: f64,
	height: f64,
	text: String,
}

impl Display for Cell {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.text)
	}
}