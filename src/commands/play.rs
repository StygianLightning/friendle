use anyhow::{bail, Result};
use serenity::utils::MessageBuilder;
use std::sync::Arc;

use serenity::client::Context;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::channel::Message;
use serenity::model::prelude::User;
use serenity::prelude::RwLock;

use crate::model::coding::{decode, Code};
use crate::model::game::{Game, GameState};
use crate::model::validate_word::validate_word;
use crate::player::PlayerState;
use crate::util::extract_second_word;
use crate::wordlist::WordList;

#[command]
#[description = "Play a round of Wordle."]
#[only_in(dm)]
pub async fn play(ctx: &Context, msg: &Message) -> CommandResult {
    if let Some(code) = extract_second_word(&msg.content) {
        match code.parse::<u32>() {
            Ok(code) => {
                if let Some(word) = decode(Code { value: code }) {
                    let player_id = msg.author.id.0;
                    let data = ctx.data.read().await;
                    let word_list = data.get::<WordList>().unwrap();
                    let player_state = data.get::<PlayerState>().unwrap();
                    let mut guard = player_state.write().await;
                    let entry = guard.games_per_player.entry(player_id);
                    if matches!(entry, std::collections::hash_map::Entry::Occupied(_)) {
                        msg.reply(ctx, "Game already in progress").await?;
                    } else {
                        match Game::new(word) {
                            Err(e) => {
                                eprintln!("{e}");
                                msg.reply(ctx, "encountered an internal problem").await?;
                            }
                            Ok(game) => {
                                entry.or_insert(game);
                                drop(guard);
                                tokio::spawn(game_loop(
                                    ctx.clone(),
                                    Arc::clone(&word_list),
                                    Arc::clone(&player_state),
                                    msg.author.clone(),
                                ));
                            }
                        }
                    }
                } else {
                    msg.reply(ctx, format!("Invalid code: {code}")).await?;
                }
            }
            Err(_) => {
                msg.reply(ctx, format!("Invalid code: {code}")).await?;
            }
        }
    } else {
        msg.reply(ctx, "Expected a code.").await?;
    }

    Ok(())
}

async fn game_loop(
    ctx: Context,
    word_list: Arc<WordList>,
    player_state: Arc<RwLock<PlayerState>>,
    user: User,
) {
    if let Err(e) = game_loop_logic(ctx, word_list, player_state, user).await {
        eprintln!("encountered error in game loop: {e}");
    }
}

async fn game_loop_logic(
    ctx: Context,
    word_list: Arc<WordList>,
    player_state: Arc<RwLock<PlayerState>>,
    user: User,
) -> Result<()> {
    user.direct_message(&ctx, |m| {
        m.content("You can now start guessing. Good luck.")
    })
    .await?;

    while let Some(msg) = user.await_reply(&ctx).await {
        let mut guard = player_state.write().await;
        let game = guard.games_per_player.get_mut(&user.id.0).unwrap();
        let guess = msg.content.to_ascii_lowercase();
        match validate_word(&guess, &word_list.words, game.solution()) {
            Err(e) => {
                eprintln!("error {e}");
                drop(guard);
                msg.reply(&ctx, "invalid word!").await?;
            }
            Ok(_) => {
                game.guess(String::from(guess), &word_list.words)?;

                let mut message_builder = MessageBuilder::new();

                for guess in game.history() {
                    message_builder.push_line_safe(format!(""));
                }

                msg.channel_id.say(&ctx, message_builder.build()).await?;
                if matches!(game.state(), GameState::Won | GameState::Lost) {
                    return Ok(());
                }
            }
        }
    }

    user.direct_message(&ctx, |m| {
        m.content("game finished early because of internal problems")
    })
    .await?;
    eprintln!("game finished early because of internal problems");
    Ok(())
}
