use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use log::error;
use prettytable::{Cell, Row, Table};
use prettytable::format::consts::FORMAT_BOX_CHARS;
use serenity::{
	framework::standard::
	StandardFramework,
	prelude::*,
};
use serenity::client::bridge::gateway::{GatewayIntents, ShardManager};
use serenity::http::Http;
use serenity::model::prelude::UserId;

use crate::classes_and_users::ClassesAndUsers;
use crate::commands::{after, before, dispatch_error, Handler, normal_message, unknown_command};
use crate::commands::*;
use crate::config::Config;
use crate::SOURCE_URLS;
use crate::substitution_pdf_getter::Weekdays;
use crate::substitution_schedule::{Substitutions, SubstitutionSchedule};

pub trait Notifier {
	fn notify_users_for_class(&self, class: &str);

	fn get_classes(&self) -> Vec<String>;

	fn insert_user(&mut self, class: String, user_id: u64);
}

#[allow(clippy::module_name_repetitions)]
pub struct DiscordNotifier {
	pub http: Arc<Http>,
	pub data: Arc<RwLock<TypeMap>>,
}

impl DiscordNotifier {
	#[allow(clippy::unreadable_literal)]
	pub async fn new(config: Config) -> Self {
		let mut owners = HashSet::new();

		owners.insert(UserId::from(191594115907977225));

		let framework = StandardFramework::new()
			.configure(|c| c
				.with_whitespace(true)
				.on_mention(Some(UserId::from(881938899876868107)))
				.prefix(config.general.prefix.as_str())
				.delimiters(vec![", ", ",", " "])
				.owners(config.general.owners.clone())
			)
			.before(before)
			.after(after)
			.unrecognised_command(unknown_command)
			.normal_message(normal_message)
			.on_dispatch_error(dispatch_error)
			.help(&MY_HELP)
			.group(&GENERAL_GROUP);

		let client_builder = Client::builder(config.general.discord_token.as_str())
			.event_handler(Handler)
			.framework(framework)
			.intents(GatewayIntents::all()); //change to only require the intents we actually want

		let mut client = client_builder.await.expect("Error creating discord client");

		{
			let mut data = client.data.write().await;
			data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
			data.insert::<Config>(config);
		}

		let http = client.cache_and_http.http.clone();
		let data = client.data.clone();


		tokio::spawn(async move {
			if let Err(why) = client.start().await {
				error!("{}", why);
			}
			std::process::exit(69);
		});

		Self {
			http,
			data,
		}
	}

	pub async fn notify_users(&self, day: Weekdays, substitutions: &SubstitutionSchedule, users_to_notify: HashSet<u64>) -> Result<(), serenity::Error> {
		let data = self.data.read().await;
		let classes_and_users = data.get::<ClassesAndUsers>().unwrap();

		for user_id in users_to_notify {
			let user = UserId::from(user_id);
			let dm_channel = user.create_dm_channel(&self.http).await?;
			let mut user_class_substitutions = HashMap::new();

			for class in classes_and_users.get_user_classes(user_id) {
				if let Some(class_substitutions) = substitutions.get_substitutions(class.as_str()) {
					user_class_substitutions.insert(class, class_substitutions);
				}
			}

			let table = Self::table_from_substitutions(&user_class_substitutions);
			dm_channel.say(
				&self.http,
				format!(
					"There are changes in schedule on {}: ```\n{}\n```Source: {}",
					day,
					table,
					SOURCE_URLS[day as usize],
				),
			).await?;
		}

		Ok(())
	}

	#[allow(clippy::needless_range_loop)]
	fn table_from_substitutions(substitutions: &HashMap<String, &Substitutions>) -> Table {
		let hour_marks = [
			"0: 07:15\n - 08:00",
			"1: 08:00\n - 09:30",
			"2: 09:50\n - 11:20",
			"3: 11:40\n - 13:10",
			"4: 13:30\n - 15:00",
			"5: 15:15\n - 16:45"
		];

		let first = substitutions.values()
			.map(|s| s.first_substitution())
			.min()
			.unwrap_or(0); // first_substitution guarantees that there is at least 1 element

		let last = substitutions.values()
			.map(|s| s.last_substitution())
			.max()
			.unwrap_or(5); // last_substitution guarantees that there is at least 1 element

		//FIXME replace table creation with table builder.
		let first_column = hour_marks[first..=last].iter()
			.map(|r| {
				Row::new(vec![Cell::new(r)])
			})
			.collect::<Vec<Row>>();

		let mut table = Table::init(first_column);
		table.insert_row(0, Row::new(vec![Cell::new("")]));
		table.set_format(*FORMAT_BOX_CHARS);

		for (class, substitution) in substitutions {
			let substitution_array = substitution.as_array();

			table.get_mut_row(0).unwrap().add_cell(Cell::new(class));

			for i in first..=last {
				let row = table.get_mut_row(i - first + 1).unwrap();
				if let Some(block) = substitution_array[i] {
					row.add_cell(Cell::new(block));
				} else {
					row.add_cell(Cell::new(""));
				}
			}
		}

		table
	}
}

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
	type Value = Arc<Mutex<ShardManager>>;
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_table_generation() {
		let mut table_map = HashMap::new();

		let mut first = Substitutions::new();
		let _ = first.block_1.insert("ONE".to_owned());
		let _ = first.block_3.insert("THREE".to_owned());
		let _ = first.block_5.insert("FIVE".to_owned());
		table_map.insert("FIRST".to_owned(), &first);

		let mut second = Substitutions::new();
		let _ = second.block_0.insert("ZERO".to_owned());
		let _ = second.block_1.insert("ONE".to_owned());
		let _ = second.block_2.insert("TWO".to_owned());
		let _ = second.block_3.insert("THREE".to_owned());
		let _ = second.block_4.insert("FOUR".to_owned());
		let _ = second.block_5.insert("FIVE".to_owned());
		table_map.insert("SECOND".to_owned(), &second);

		let out = DiscordNotifier::table_from_substitutions(&table_map);

		let expected_1 = "\
		┌──────────┬────────┬───────┐\n\
		│          │ SECOND │ FIRST │\n\
		├──────────┼────────┼───────┤\n\
		│ 0: 07:15 │ ZERO   │       │\n\
		│  - 08:00 │        │       │\n\
		├──────────┼────────┼───────┤\n\
		│ 1: 08:00 │ ONE    │ ONE   │\n\
		│  - 09:30 │        │       │\n\
		├──────────┼────────┼───────┤\n\
		│ 2: 09:50 │ TWO    │       │\n\
		│  - 11:20 │        │       │\n\
		├──────────┼────────┼───────┤\n\
		│ 3: 11:40 │ THREE  │ THREE │\n\
		│  - 13:10 │        │       │\n\
		├──────────┼────────┼───────┤\n\
		│ 4: 13:30 │ FOUR   │       │\n\
		│  - 15:00 │        │       │\n\
		├──────────┼────────┼───────┤\n\
		│ 5: 15:15 │ FIVE   │ FIVE  │\n\
		│  - 16:45 │        │       │\n\
		└──────────┴────────┴───────┘\n";

		let expected_2 = "\
		┌──────────┬───────┬────────┐\n\
		│          │ FIRST │ SECOND │\n\
		├──────────┼───────┼────────┤\n\
		│ 0: 07:15 │       │ ZERO   │\n\
		│  - 08:00 │       │        │\n\
		├──────────┼───────┼────────┤\n\
		│ 1: 08:00 │ ONE   │ ONE    │\n\
		│  - 09:30 │       │        │\n\
		├──────────┼───────┼────────┤\n\
		│ 2: 09:50 │       │ TWO    │\n\
		│  - 11:20 │       │        │\n\
		├──────────┼───────┼────────┤\n\
		│ 3: 11:40 │ THREE │ THREE  │\n\
		│  - 13:10 │       │        │\n\
		├──────────┼───────┼────────┤\n\
		│ 4: 13:30 │       │ FOUR   │\n\
		│  - 15:00 │       │        │\n\
		├──────────┼───────┼────────┤\n\
		│ 5: 15:15 │ FIVE  │ FIVE   │\n\
		│  - 16:45 │       │        │\n\
		└──────────┴───────┴────────┘\n";

		assert!(out.to_string() == expected_1 || out.to_string() == expected_2);
	}

	#[test]
	fn test_table_generation_2() {
		let mut table_map = HashMap::new();

		let mut first = Substitutions::new();
		let _ = first.block_1.insert("ONE".to_owned());
		let _ = first.block_4.insert("FOUR".to_owned());
		table_map.insert("FIRST".to_owned(), &first);

		let mut second = Substitutions::new();
		let _ = second.block_3.insert("THREE".to_owned());
		table_map.insert("SECOND".to_owned(), &second);

		let out = DiscordNotifier::table_from_substitutions(&table_map);

		let expected_1 = "\
		┌──────────┬────────┬───────┐\n\
		│          │ SECOND │ FIRST │\n\
		├──────────┼────────┼───────┤\n\
		│ 1: 08:00 │        │ ONE   │\n\
		│  - 09:30 │        │       │\n\
		├──────────┼────────┼───────┤\n\
		│ 2: 09:50 │        │       │\n\
		│  - 11:20 │        │       │\n\
		├──────────┼────────┼───────┤\n\
		│ 3: 11:40 │ THREE  │       │\n\
		│  - 13:10 │        │       │\n\
		├──────────┼────────┼───────┤\n\
		│ 4: 13:30 │        │ FOUR  │\n\
		│  - 15:00 │        │       │\n\
		└──────────┴────────┴───────┘\n";

		let expected_2 = "\
		┌──────────┬───────┬────────┐\n\
		│          │ FIRST │ SECOND │\n\
		├──────────┼───────┼────────┤\n\
		│ 1: 08:00 │ ONE   │        │\n\
		│  - 09:30 │       │        │\n\
		├──────────┼───────┼────────┤\n\
		│ 2: 09:50 │       │        │\n\
		│  - 11:20 │       │        │\n\
		├──────────┼───────┼────────┤\n\
		│ 3: 11:40 │       │ THREE  │\n\
		│  - 13:10 │       │        │\n\
		├──────────┼───────┼────────┤\n\
		│ 4: 13:30 │ FOUR  │        │\n\
		│  - 15:00 │       │        │\n\
		└──────────┴───────┴────────┘\n";

		assert!(out.to_string() == expected_1 || out.to_string() == expected_2);
	}
}