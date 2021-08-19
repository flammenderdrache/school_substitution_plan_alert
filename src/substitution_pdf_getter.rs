use std::time::Duration;
use chrono::Weekday;
use reqwest::Client;
use std::fmt::{Display, Formatter};

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

pub struct SubstitutionPDFGetter<'a> {
	urls: [&'a str; 5],
	client: Client,
}

impl<'a> SubstitutionPDFGetter<'a> {
	pub fn new(client: Client) -> Self {
		Self {
			urls: [
				"https://buessing.schule/plaene/VertretungsplanA4_Montag.pdf",
				"https://buessing.schule/plaene/VertretungsplanA4_Dienstag.pdf",
				"https://buessing.schule/plaene/VertretungsplanA4_Mittwoch.pdf",
				"https://buessing.schule/plaene/VertretungsplanA4_Donnerstag.pdf",
				"https://buessing.schule/plaene/VertretungsplanA4_Freitag.pdf",
			],
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