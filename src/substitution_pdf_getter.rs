use std::fmt::{Display, Formatter};
use std::time::Duration;

use chrono::{Weekday, Datelike};
use reqwest::Client;
use num_traits::PrimInt;

use crate::SOURCE_URLS;
use std::convert::{TryFrom, TryInto};

///Enum with the weekdays where a Substitution PDF is available
#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
pub enum Weekdays {
	Monday = 0,
	Tuesday = 1,
	Wednesday = 2,
	Thursday = 3,
	Friday = 4,
}

impl Weekdays {
	//It is not &self, just self here due to https://rust-lang.github.io/rust-clippy/master/index.html#trivially_copy_pass_by_ref
	//Thank clippy :p
	pub fn next_day(self) -> Self {
		match self {
			Weekdays::Monday => Weekdays::Tuesday,
			Weekdays::Tuesday => Weekdays::Wednesday,
			Weekdays::Wednesday => Weekdays::Thursday,
			Weekdays::Thursday => Weekdays::Friday,
			Weekdays::Friday => Weekdays::Monday,
		}
	}

	pub fn today() -> Self {
		Self::from(chrono::Local::today().weekday())
	}
}

impl Display for Weekdays {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let self_as_string = match self {
			Weekdays::Monday => "Monday",
			Weekdays::Tuesday => "Tuesday",
			Weekdays::Wednesday => "Wednesday",
			Weekdays::Thursday => "Thursday",
			Weekdays::Friday => "Friday",
		};

		write!(f, "{}", self_as_string)
	}
}

impl ToString for Weekdays {
	fn to_string(&self) -> String {
		let mut day = format!("{}", self);
		day.to_lowercase();

		day
	}
}

impl From<Weekday> for Weekdays {
	fn from(day: Weekday) -> Self {
		match day {
			Weekday::Tue => Weekdays::Tuesday,
			Weekday::Wed => Weekdays::Wednesday,
			Weekday::Thu => Weekdays::Thursday,
			Weekday::Fri => Weekdays::Friday,
			_ => Weekdays::Monday,
		}
	}
}

impl TryFrom<u8> for Weekdays {
	type Error = ();

	fn try_from(day: u8) -> Result<Self, Self::Error> {
		match day {
			0 => Ok(Weekdays::Monday),
			1 => Ok(Weekdays::Tuesday),
			2 => Ok(Weekdays::Wednesday),
			3 => Ok(Weekdays::Thursday),
			4 => Ok(Weekdays::Friday),
			_ => Err(()),
		}
	}
}

impl<T: Into<Weekdays>> TryFrom<T> for Weekdays {
	type Error = ();

	fn try_from(string: T) -> Result<Self, Self::Error> {
		let mut day_string = string.to_string().as_str();
		day_string.make_ascii_lowercase();

		let mut levenshteine: [u8; 5] = [1; 5];
		let mut day = Weekdays::Monday;

		//consider implementing the iter trait here
		for i in 0..5 {
			levenshteine[i] = levenshtein::levenshtein(day_string, &day.to_string()) as u8;
			day = day.next_day();
		}

		levenshteine.iter().min()
			.filter(|distance| (*distance < 5 as &u8))
			//unwrap is safe, because
			.map(|day| Weekdays::try_from(day).ok())
			.flatten()
			.ok_or(())
	}
}

pub struct SubstitutionPDFGetter<'a> {
	urls: [&'a str; 5],
	client: Client,
}

impl<'a> SubstitutionPDFGetter<'a> {
	pub fn new(client: Client) -> Self {
		Self {
			urls: SOURCE_URLS,
			client,
		}
	}

	///Returns an instance of self with a default client
	pub fn default() -> Self {
		let client = Client::builder()
			.connect_timeout(Duration::from_secs(20))
			.timeout(Duration::from_secs(20))
			.build()
			.unwrap();

		Self::new(
			client
		)
	}

	/// Returns result with an Err or a Vector with the binary data of the request-response
	/// Does not check if the response is valid, this is the responsibility of the caller.
	pub async fn get_weekday_pdf(&self, day: Weekdays) -> Result<Vec<u8>, reqwest::Error> {
		let url = self.urls[day as usize];
		let request = self.client
			.get(url)
			.header("Authorization", "Basic aGJzdXNlcjpoYnNwYXNz")
			.build()
			.unwrap();

		let response = self.client.execute(request).await?;
		let bytes = response.bytes().await?;

		Ok(bytes.to_vec())
	}
}