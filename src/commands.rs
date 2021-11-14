use std::collections::HashSet;

use log::{debug, error, info};
use serenity::{
	framework::standard::{
		CommandResult,
		DispatchError,
		macros::{command, group, help, hook},
	},
	model::channel::Message,
	prelude::*,
};
use serenity::async_trait;
use serenity::framework::standard::{Args, CommandGroup, help_commands, HelpOptions};
use serenity::model::prelude::{Activity, OnlineStatus, Ready, UserId};

use crate::{Data, DataStore};
use crate::classes_and_users::ClassesAndUsers;
use crate::util::sanitize_and_check_register_class_input;

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
	let datastore = data.get::<Data>().unwrap();

	let class_whitelist = datastore.get_class_whitelist().expect("Error getting class whitelist");
	if !class_whitelist.contains(&class) {
		msg.reply(&ctx.http, "Sorry but the specified class is not on the whitelist. Please contact us to request it getting put on the whitelist").await?;
		return Ok(());
	}

	let classes_and_users = data.get_mut::<ClassesAndUsers>().unwrap();
	let _ = classes_and_users.insert_user(class.clone(), user);

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
#[aliases("classes", "list_classes", "list", "show")]
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
	let success = classes_and_users.remove_user_from_class(class.as_str(), user).unwrap_or(false);
	if !success {
		msg.reply_ping(&ctx.http, "An error occurred adding you to the class notifications").await?;
	}

	msg.reply_ping(&ctx.http, format!("Removed you from class {}", &class)).await?;
	info!("Unregistered {}#{} from class {}", msg.author.name, msg.author.discriminator, &class);


	Ok(())
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