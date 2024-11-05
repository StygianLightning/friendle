use crate::model::validate_word::validate_word_format;
use crate::util::extract_second_word;
use crate::wordlist::WordList;
use serenity::client::Context;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::channel::Message;

#[command]
#[description = "Encode a word as a secret for Wordle."]
#[only_in(dm)]
pub async fn encode(ctx: &Context, msg: &Message) -> CommandResult {
    if let Some(word) = extract_second_word(&msg.content) {
        validate_encode_and_post(ctx, msg, word).await?;
    } else {
        msg.reply(ctx, "Please provide a word.").await?;
    }
    Ok(())
}

pub async fn validate_encode_and_post (ctx: &Context, msg: &Message, word: &str) -> CommandResult {

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

                let data = ctx.data.read().await;
                let word_list = data.get::<WordList>().unwrap();
                let mut reply = String::from(
                    "To play a game of Wordle with your secret word, use the following command.\n",
                );
                if !word_list.words.contains(word) {
                    reply += "Note that your word is not in the original Wordle word list.";
                };

                msg.reply(ctx, reply).await?;
                msg.reply(ctx, format!(".play `{value}`")).await?;
            }
        }
    Ok(())
}
