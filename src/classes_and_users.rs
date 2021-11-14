use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::Arc;

use log::debug;

use crate::{Data, DataStore, TypeMapKey};

//Maybe accept something that implements datastore for reading and writing
pub struct ClassesAndUsers {
	datastore: Arc<Data>,
	classes_and_users: HashMap<String, HashSet<u64>>,
}

impl TypeMapKey for ClassesAndUsers {
	type Value = ClassesAndUsers;
}

impl ClassesAndUsers {
	pub fn new(datastore: Arc<Data>) -> Self {
		let classes_and_users = datastore.get_classes_and_users().unwrap_or_default();

		Self {
			datastore,
			classes_and_users,
		}
	}

	pub fn save(&self) -> Result<(), Box<dyn Error>> {
		self.datastore.store_classes_and_users(&self.classes_and_users)
	}

	#[allow(clippy::or_fun_call)]
	pub fn insert_user(&mut self, class: String, user_id: u64) -> Result<(), Box<dyn Error>> {
		self.
			classes_and_users
			.entry(class)
			.or_insert(HashSet::new())
			.insert(user_id);
		self.save()
	}

	/// Returns a boolean of whether the operation was successful.
	pub fn remove_user_from_class(&mut self, class: &str, user_id: u64) -> Result<bool, Box<dyn Error>> {
		debug!("Class for user {} is {}", class, &user_id);
		let mut successful = false;
		if let Some(class_users) = self.classes_and_users.get_mut(class) {
			successful = class_users.remove(&user_id);
			if class_users.is_empty() {
				self.classes_and_users.remove(class);
			}
		}

		self.save()?;
		Ok(successful)
	}

	/// Gets the classes a user subscribed to.
	pub fn get_user_classes(&self, user_id: u64) -> Vec<String> {
		let mut classes = Vec::new();
		let classes_and_users = &self.classes_and_users;

		for (class, user_ids) in classes_and_users {
			if user_ids.contains(&user_id) {
				classes.push(class.clone());
			}
		}

		classes
	}

	pub fn _get_classes(&self) -> HashSet<String> {
		let mut classes = HashSet::new();
		for class in self.classes_and_users.keys() {
			classes.insert(class.clone());
		}
		classes
	}

	pub fn get_inner_classes_and_users(&self) -> &HashMap<String, HashSet<u64>> {
		&self.classes_and_users
	}
}

#[cfg(test)]
mod tests {
	use crate::data::tests::get_temp_data;

	use super::*;

	#[test]
	fn test_insert_get_and_remove_user() {
		let datastore = Arc::new(get_temp_data());
		let mut classes_and_users = ClassesAndUsers::new(datastore.clone());

		let class = "TEST";
		let class_2 = "TEST2";

		classes_and_users.insert_user(class.to_owned(), 1).unwrap();
		classes_and_users.insert_user(class.to_owned(), 2).unwrap();
		classes_and_users.insert_user(class_2.to_owned(), 1).unwrap();
		classes_and_users.insert_user(class_2.to_owned(), 3).unwrap();

		assert_eq!(classes_and_users.get_user_classes(1), vec![class.to_owned(), class_2.to_owned()]);
		assert_eq!(classes_and_users.get_user_classes(2), vec![class.to_owned()]);
		assert_eq!(classes_and_users.get_user_classes(3), vec![class_2.to_owned()]);

		classes_and_users.remove_user_from_class(class, 1).unwrap();

		assert_eq!(classes_and_users.get_user_classes(1), vec![class_2.to_owned()]);
	}
}