use crate::model::validate_word::validate_word_format;
use serenity::client::Context;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::channel::Message;

#[command]
#[description = "Encode a word as a secret for Wordle."]
#[only_in(dm)]
pub async fn encode(ctx: &Context, msg: &Message) -> CommandResult {
    println!("encode called, msg: {msg:?}");
    if let Some(word) = msg.content.split_ascii_whitespace().skip(1).take(1).next() {
        match validate_word_format(word) {
            Err(_) => {
                msg.reply(ctx, format!("Invalid word format: {word}. Only five-letter alphanumeric words are supported."))
                    .await?;
            }
            Ok(_) => {
                let code = crate::model::coding::encode(word);
                let value = code.value;
                msg.reply(
                    ctx,
                    format!("To play a game of Wordle with your secret word, use `.play {value}`"),
                )
                .await?;
            }
        }
    } else {
        msg.reply(ctx, "Please provide a word.").await?;
    }
    Ok(())
}
