use crate::buttons::copy_result_button::CopyResultButton;


use crate::buttons::show_keyboard_button::ShowKeyboardButton;
use crate::model::evaluation::EmojiMode;
use crate::model::game::{GameState};
use crate::player::PlayerState;

use crate::wordlist::WordList;

use serenity::framework::standard::macros::hook;
use serenity::model::channel::Message;

use serenity::prelude::Context;
use serenity::utils::MessageBuilder;

#[hook]
pub async fn message_hook(ctx: &Context, msg: &Message) {
    // In theory, we could use serenity's collector feature to
    // await the next response in the channel where the game was started
    // instead of using a message hook; however, when I tested it,
    // callbacks were never actually executed that way.
    if let Err(err) = handle_message(ctx, msg).await {
        eprintln!("Encountered error in game loop: {err}");
    }
}

async fn handle_message(ctx: &Context, msg: &Message) -> anyhow::Result<()> {
    let data = ctx.data.read().await;

    if msg.content.starts_with('.') {
        // ignore commands
        return Ok(());
    }

    if msg.guild_id.is_some() {
        // ideally, this reply should be ephemeral, but ephemeral messages are restricted to interaction responses
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

    match game_state {
        // Reply with an extra message if the game is now finished.
        GameState::Lost => {
            let solution = game.solution();
            // TODO add extra loss messages and select one at random for fun
            msg.reply(
                &ctx,
                format!("Unfortunately, you're out of tries. The solution was ||`{solution}`||"),
            )
            .await?;
        }
        GameState::Won => {
            // TODO add extra win messages and select one at random for fun
            msg.reply(&ctx, String::from("You won! Good job :)"))
                .await?;
        }
        GameState::InProgress => {
            // no action
        }
    }

    game.display_game_state_header(&mut message_builder);
    game.display_state(&mut message_builder, EmojiMode::Unicode);

    msg.channel_id
        .send_message(&ctx, |m| {
            m.content(message_builder);
            match game_state {
                GameState::InProgress => {
                    m.components(|comps| {
                        comps.create_action_row(|row| row.add_button(ShowKeyboardButton::button()));
                        comps
                    });
                }
                _ => {
                    m.components(|comps| {
                        comps.create_action_row(|row| row.add_button(CopyResultButton::button()));
                        comps
                    });
                }
            }
            m
        })
        .await?;

    {
        // Get the game state again and update it.
        // Always keep the game state until a new game is started, even if it's already finished.
        // This allows us to run button commands to copy the game in a format a user can copy paste to share their results.
        let mut lock = player_state.lock().unwrap();
        lock.games_per_player.insert(user.id.0, game);
    }

    Ok(())
}
