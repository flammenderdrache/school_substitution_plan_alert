use reqwest::Client;
use std::io::Write;
use std::time::Duration;

pub enum Weekdays {
	Monday = 0,
	Tuesday = 1,
	Wednesday = 2,
	Thursday = 3,
	Friday = 4,
}

pub struct SubstitutionPDFGetter<'a> {
	urls: [&'a str;5],
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
	pub async fn get_weekday_pdf(&self, day: Weekdays) -> Result<Vec<u8>, reqwest::Error>{
		let url = self.urls[day as usize];
		let request = self.client
			.get(url)
			.header("Authorization", "Basic aGJzdXNlcjpoYnNwYXNz")
			.build()
			.unwrap();

		let response = self.client.execute(request).await?;
		let mut bytes = response.bytes().await?;


		Ok(bytes.to_vec())
	}
}