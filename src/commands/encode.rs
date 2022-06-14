use crate::model::validate_word::validate_word_format;
use crate::util::extract_second_word;
use serenity::client::Context;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::channel::Message;

#[command]
#[description = "Encode a word as a secret for Wordle."]
#[only_in(dm)]
pub async fn encode(ctx: &Context, msg: &Message) -> CommandResult {
    if let Some(word) = extract_second_word(&msg.content) {
        match validate_word_format(word) {
            Err(_) => {
                msg.reply(
                    ctx,
                    format!("Invalid word format: {word}. Only five-letter words with letters a-z are supported."),
                )
                .await?;
            }
            Ok(_) => {
                let code = crate::model::coding::encode(word);
                let value = code.value;

                // TODO check if the word is in the supported word list. If it isn't, send an extra warning message!
                msg.reply(
                    ctx,
                    "To play a game of Wordle with your secret word, use the following command:",
                )
                .await?;
                msg.reply(ctx, format!("`.play {value}`")).await?;
            }
        }
    } else {
        msg.reply(ctx, "Please provide a word.").await?;
    }
    Ok(())
}
