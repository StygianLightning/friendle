use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use std::collections::HashSet;

use serenity::framework::standard::{
    help_commands, macros::help, Args, CommandGroup, CommandResult, HelpOptions,
};

#[help]
#[individual_command_tip = "Pass in the name of a command to learn more about it."]
#[command_not_found_text = "Unknown command: `{}`."]
#[max_levenshtein_distance(3)]
#[lacking_permissions = "Hide"]
// Currently, commands can be executed by anyone, so we disable the strikethrough explanation
#[strikethrough_commands_tip_in_guild = ""]
// However, commands are currently limited to guilds, so we include the strikethrough explanation in DMs
#[strikethrough_commands_tip_in_dm = "~~`Strikethrough commands`~~ are only available in channels."]
async fn my_help(
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
