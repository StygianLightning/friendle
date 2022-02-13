
use serenity::client::Context;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::channel::Message;

#[command]
#[description = "Get the secret code to play a round of Wordle."]
#[only_in(dm)]
pub async fn play(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "not yet implemented").await?;
    Ok(())
}
