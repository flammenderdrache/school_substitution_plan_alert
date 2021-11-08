use std::fmt::{Display, Formatter};
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone)]
pub struct StringError {
	message: String,
}

impl StringError {
	pub fn new(message: &str) -> Self {
		message.into()
	}
}

impl Display for StringError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f,"{}", self.message)
	}
}

impl From<&str> for StringError {
	fn from(message: &str) -> Self {
		Self {
			message: message.to_owned(),
		}
	}
}

impl From<String> for StringError {
	fn from(message: String) -> Self {
		Self {
			message
		}
	}
}

impl std::error::Error for StringError {}