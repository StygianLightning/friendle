use serenity::client::Context;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::channel::Message;

#[command]
#[description = "Play a round of Wordle."]
pub async fn play(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "not yet implemented").await?;
    Ok(())
}
