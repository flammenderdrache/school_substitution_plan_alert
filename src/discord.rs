use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use serenity::{
	framework::standard::{
		CommandResult,
		DispatchError,
		macros::{command, group, help, hook},
		StandardFramework,
	},
	model::channel::Message,
	prelude::*,
};
use serenity::async_trait;
use serenity::client::bridge::gateway::{GatewayIntents, ShardManager};
use serenity::framework::standard::{Args, CommandGroup, help_commands, HelpOptions};
use serenity::http::Http;
use serenity::model::prelude::{Activity, OnlineStatus, Ready, UserId};
use sqlx::{Pool, Sqlite};

use crate::substitution_pdf_getter::Weekdays;
use crate::config::Config;
use crate::USER_AND_CLASSES_SAVE_LOCATION;

#[derive(Serialize, Deserialize)]
pub struct ClassesAndUsers {
	classes_and_users: HashMap<String, HashSet<u64>>,
}

impl ClassesAndUsers {
	pub fn default() -> Self {
		Self {
			classes_and_users: HashMap::new()
		}
	}

	pub fn new_from_file(path: &Path) -> Self {
		if !path.exists() {
			return Self::default();
		}

		let file = std::fs::OpenOptions::new()
			.read(true)
			.write(false)
			.open(path)
			.expect("Couldn't open user file");

		serde_json::from_reader(file).expect("Malformed User Save file")
	}

	//TODO make function Async and use Tokio async file operations
	pub fn write_to_file(&self, path: &Path) {
		let mut file = std::fs::OpenOptions::new()
			.write(true)
			.create(true)
			.truncate(true)
			.open(path)
			.expect("Couldn't open user save file");
		let json = serde_json::to_string_pretty(self).unwrap();
		if let Err(why) = file.write_all(json.as_bytes()) {
			error!("{}", why);
		}
	}

	#[allow(clippy::or_fun_call)]
	pub fn insert_user(&mut self, class: String, user_id: u64, user_and_classes_save_location: &Path) {
		self.
			classes_and_users
			.entry(class)
			.or_insert(HashSet::new())
			.insert(user_id);
		self.write_to_file(user_and_classes_save_location);
	}

	//maybe return result instead of bool
	pub fn remove_user_from_class(&mut self, class: &str, user_id: u64, user_and_classes_save_location: &Path) -> bool {
		debug!("Class for user {} is {}", class, &user_id);
		let mut successful = false;
		if let Some(class_users) = self.classes_and_users.get_mut(class) {
			successful = class_users.remove(&user_id);
			if class_users.is_empty() {
				self.classes_and_users.remove(class);
			}
		}
		self.write_to_file(user_and_classes_save_location);

		successful
	}

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

	pub fn get_classes(&self) -> Vec<String> {
		let mut classes = Vec::new();
		for class in self.classes_and_users.keys() {
			classes.push(class.clone());
		}
		classes
	}
}

impl TypeMapKey for ClassesAndUsers {
	type Value = ClassesAndUsers;
}

pub trait Notifier {
	fn notify_users_for_class(&self, class: &str);

	fn get_classes(&self) -> Vec<String>;

	fn insert_user(&mut self, class: String, user_id: u64);
}

#[allow(clippy::module_name_repetitions)]
pub struct DiscordNotifier {
	http: Arc<Http>,
	data: Arc<RwLock<TypeMap>>,
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
				.owners(owners)
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
			data.insert::<ClassesAndUsers>(ClassesAndUsers::new_from_file(Path::new(USER_AND_CLASSES_SAVE_LOCATION)));
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

	pub async fn notify_users_for_class(&self, class: &str, day: Weekdays) -> Result<(), serenity::Error> {
		info!("Notifying all users in class {} on day {}", class, day);

		let data = self.data.read().await;
		let classes_and_users = data.get::<ClassesAndUsers>().unwrap();

		for user in classes_and_users.classes_and_users.get(class).unwrap() {
			let user = UserId::from(*user);
			let dm_channel = user.create_dm_channel(&self.http).await?;
			dm_channel.say(&self.http, format!(
				"There is a change in schedule on {} for class {}",
				day,
				class,
			),
			).await?;//TODO refine, send the link to the corresponding day maybe too etc.
		}

		Ok(())
	}

	pub async fn get_classes(&self) -> Vec<String> {
		let data = self.data.read().await;
		let classes_and_users = data.get::<ClassesAndUsers>().unwrap();
		classes_and_users.get_classes()
	}

	// pub async fn insert_user(&mut self, class: String, user_id: u64) {
	// 	let mut data = self.data.write().await;
	// 	let classes_and_users = data.get_mut::<ClassesAndUsers>().unwrap();
	// 	classes_and_users.insert_user(class, user_id);
	// }
}

#[group]
#[commands(register, show_classes, unregister)]
pub struct General;

#[command]
#[aliases("register_class")]
#[description("Subscribes you to notifications for a specific class.")]
#[example("BGYM191")]
#[example("FOS201")]
async fn register(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let user = msg.author.id.0;
	let mut class = args.single::<String>().unwrap();

	let sanitized_input = sanitize_and_check_register_class_input(class.as_str());
	match sanitized_input {
		Ok(sanitized_input_class) => { class = sanitized_input_class; }
		Err(why) => {
			msg.reply_ping(&ctx.http, why).await?;
			return Ok(());
		}
	}

	let mut data = ctx.data.write().await;
	let classes_and_users = data.get_mut::<ClassesAndUsers>().unwrap();
	classes_and_users.insert_user(class.clone(), user, Path::new(USER_AND_CLASSES_SAVE_LOCATION));

	msg.reply_ping(&ctx.http, format!(
		"Registered you for class {}.\n \
		You will receive updates in the future.\n\
		_Note that you might not receive an update for today or tomorrow if it was published before you registered._",
		&class
	)).await?;
	info!("Registered {}#{} for class {}", msg.author.name, msg.author.discriminator, &class);

	Ok(())
}

#[command]
#[aliases("classes", "list_classes", "list")]
#[description("Lists all the classes whose notifications you subscribed to.")]
async fn show_classes(ctx: &Context, msg: &Message) -> CommandResult {
	let user = msg.author.id.0;
	let channel = msg.channel_id;
	let data = ctx.data.read().await;
	let classes_and_users = data.get::<ClassesAndUsers>().unwrap();
	let classes = classes_and_users.get_user_classes(user);

	channel.send_message(&ctx.http, |msg|
		msg.embed(|embed| {
			embed.description(
				if classes.is_empty() {
					"You haven't registered for updates for any class".to_owned()
				} else {
					classes.join("\n")
				}
			)
		}),
	).await?;

	Ok(())
}

#[command]
#[aliases("remove", "delete")]
#[description("Removes your subscription to notifications for a specific class.")]
#[example("BGYM191")]
#[example("FOS201")]
async fn unregister(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let user = msg.author.id.0;
	let class = args.single::<String>().unwrap();
	if class.len() < 3 {
		msg.reply_ping(&ctx.http, "Incorrect Arguments").await?;
		return Ok(());
	}
	let class = class.to_uppercase();

	let mut data = ctx.data.write().await;
	let classes_and_users = data.get_mut::<ClassesAndUsers>().unwrap();
	let success = classes_and_users.remove_user_from_class(class.as_str(), user, Path::new(USER_AND_CLASSES_SAVE_LOCATION));
	if !success {
		msg.reply_ping(&ctx.http, "An error occurred adding you to the class notifications").await?;
	}

	msg.reply_ping(&ctx.http, format!("Removed you from class {}", &class)).await?;
	info!("Registered {}#{} for class {}", msg.author.name, msg.author.discriminator, &class);


	Ok(())
}

pub struct ConnectionPool;

impl TypeMapKey for ConnectionPool {
	type Value = Pool<Sqlite>;
}

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
	type Value = Arc<Mutex<ShardManager>>;
}

#[hook]
pub async fn before(_ctx: &Context, msg: &Message, command_name: &str) -> bool {
	info!("Got command '{}' by user '{}'", command_name, msg.author.name);

	true // if `before` returns false, command processing doesn't happen.
}

#[hook]
pub async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
	match command_result {
		Ok(()) => info!("Processed command '{}'", command_name),
		Err(why) => error!("Command returned an error: {:?}", why),
	}
}

#[hook]
pub async fn unknown_command(ctx: &Context, msg: &Message, unknown_command_name: &str) {
	debug!("Could not find command named '{}'\n(Message content: \"{}\")", unknown_command_name, msg.content);
	let reply = msg.channel_id.say(
		&ctx.http,
		format!(
			"Sorry, couldn't find a command named '`{}`'\n\n With the `help` command you can list all available commands",
			unknown_command_name),
	).await;

	if let Err(why) = reply {
		error!("Error replying to unknown command: {:?}", why);
	}
}

#[hook]
pub async fn normal_message(_ctx: &Context, msg: &Message) {
	info!("Processed non Command message: '{}'", msg.content)
}

#[hook]
pub async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
	if let DispatchError::Ratelimited(info) = error {
		// We notify them only once.
		if info.is_first_try {
			let _ = msg
				.channel_id
				.say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
				.await;
		}
	}
}

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
	// async fn message(&self, ctx: Context, message: Message) {
	// 	info!("Message by: {} with content: {}", message.author.name, message.content);
	// }

	async fn ready(
		&self,
		ctx: Context,
		data_about_bot: Ready,
	) {
		info!("{} está aqui!", data_about_bot.user.name);
		let activity = Activity::watching("the substitution plan | ~help for help");
		ctx.set_presence(Some(activity), OnlineStatus::Online).await;
	}
}

#[help]
#[individual_command_tip = "If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find command `{}`."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Hide"]
#[wrong_channel = "Nothing"]
#[strikethrough_commands_tip_in_dm = ""]
#[strikethrough_commands_tip_in_guild = ""]
pub async fn my_help(
	context: &Context,
	msg: &Message,
	args: Args,
	help_options: &'static HelpOptions,
	groups: &[&'static CommandGroup],
	owners: HashSet<UserId>,
) -> CommandResult {
	let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
	Ok(())
}

///Removes the dots to make e.g. "BGYM19.1" valid (turning it into "BGYM191")
///Also turns the input uppercase; "BGym19.1" -> "BGYM191" as that is how they are referred to in the PDF
fn sanitize_and_check_register_class_input(input: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
	let input = input.replace('.', "");

	if input.len() < 4 {
		return Err("Argument too short".into())
	}

	if !(input.contains(char::is_alphabetic) &&
		input.contains(char::is_ascii_digit)) {
		return Err("Argument is incorrectly formatted".into())
	}

	let input = input.to_uppercase();

	Ok(input)
}


#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_sanitize_should_pass() {
		let test_class = "BGYM191";
		let output = sanitize_and_check_register_class_input(test_class).unwrap();
		assert_eq!(output, test_class);
	}

	#[test]
	#[should_panic]
	fn test_sanitize_too_short() {
		let test_class = "B2";
		let _ = sanitize_and_check_register_class_input(test_class).unwrap();
	}

	#[test]
	fn test_sanitize_remove_dots() {
		let test_class = "BGYM19.1";
		let output = sanitize_and_check_register_class_input(test_class).unwrap();
		assert_eq!(output, "BGYM191");
	}

	#[test]
	#[should_panic]
	fn test_sanitize_missing_class_number() {
		let test_class = "ELIAS";
		let _ = sanitize_and_check_register_class_input(test_class).unwrap();
	}

	#[test]
	#[should_panic]
	fn test_sanitize_only_numbers() {
		let test_class = "1234567420";
		let _ = sanitize_and_check_register_class_input(test_class).unwrap();
	}

	#[test]
	#[should_panic]
	fn test_sanitize_check_between_large_char_and_small_char_ascii_value() {
		let test_class = "BGY/@;19[1";
		let output = sanitize_and_check_register_class_input(test_class).unwrap();
		assert_eq!(output, "BGYM191")
	}
}