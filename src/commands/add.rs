use serenity::client::Context;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::channel::Message;

#[command]
#[description = "Add a word."]
pub async fn add(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "not yet implemented").await?;
    Ok(())
}
