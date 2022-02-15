use crate::constants;
use crate::model::evaluation::get_emoji;
use crate::model::game::GameState;
use crate::player::PlayerState;
use crate::util::get_regional_indicator;
use crate::wordlist::WordList;
use serenity::framework::standard::macros::hook;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use serenity::utils::MessageBuilder;

#[hook]
pub async fn message_hook(ctx: &Context, msg: &Message) {
    if let Err(err) = handle_message(ctx, msg).await {
        eprintln!("Encountered error in game loop: {err}");
    }
}

async fn handle_message(ctx: &Context, msg: &Message) -> anyhow::Result<()> {
    let data = ctx.data.read().await;

    if msg.content.starts_with(".") {
        // ignore commands
        return Ok(());
    }

    if msg.guild_id.is_some() {
        msg.reply(ctx, "Guesses are only accepted in DMs").await?;
        return Ok(());
    }

    let user = &msg.author;

    let word_list = data.get::<WordList>().unwrap();
    let player_state = data.get::<PlayerState>().unwrap();

    let game = {
        let mut lock = player_state.lock().unwrap();
        lock.games_per_player.get_mut(&user.id.0).cloned()
    };

    if game.is_none() {
        msg.reply(ctx, "No game in progress.").await?;
        return Ok(());
    }

    let mut game = game.unwrap();
    let guess = msg.content.to_ascii_lowercase();

    if let Err(err) = game.guess(guess, &word_list.words) {
        msg.reply(ctx, format!("{err}")).await?;
        return Ok(());
    }

    let game_state = game.state();

    let mut message_builder = MessageBuilder::new();

    let code = game.code().value;
    match game_state {
        GameState::Lost => {
            let solution = game.solution();
            msg.reply(
                &ctx,
                format!("Unfortunately, you're out of tries. The solution was ||`{solution}`||"),
            )
            .await?;
            let line = format!("X/{}", constants::MAX_GUESSES);
            message_builder.push_line(format!("Friendle `{code}`: {line}"));
        }
        GameState::Won => {
            msg.reply(&ctx, format!("You won! Good job :)")).await?;
            let line = format!("{}/{}", game.history().len(), constants::MAX_GUESSES);
            message_builder.push_line(format!("Friendle `{code}`: {line}"));
        }
        GameState::InProgress => {
            message_builder.push_line(format!("Friendle `{code}`"));
            message_builder.push_line(format!(
                "{}/{} [in progress]",
                game.history().len(),
                crate::constants::MAX_GUESSES
            ));
        }
    }

    for guess in game.history() {
        if game_state == GameState::InProgress {
            // guessed word converted to emojis
            message_builder.push_line(String::from_iter(
                guess
                    .word
                    .chars()
                    // add a zero-width space unicode character after each emoji to prevent Serenity from merging successive emojis.
                    .map(|c| format!("{}\u{200c}", get_regional_indicator(c))),
            ));
        }
        // evaluation converted to emojis
        message_builder.push_line_safe(String::from_iter(
            guess
                .evaluation
                .iter()
                .map(|eval| format!("{}", get_emoji(*eval))),
        ));
        if game_state == GameState::InProgress {
            message_builder.push_line_safe("");
        }
    }

    msg.channel_id.say(&ctx, message_builder.build()).await?;

    {
        // get the game state again and update it -- either write the new game state if it's still in progress or remove it if it's finished
        let mut lock = player_state.lock().unwrap();
        if game_state == GameState::InProgress {
            lock.games_per_player.insert(user.id.0, game);
        } else {
            lock.games_per_player.remove(&user.id.0);
        }
    }

    Ok(())
}
