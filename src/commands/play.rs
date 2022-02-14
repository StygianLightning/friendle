use crate::constants;
use crate::model::coding::{decode, Code};
use crate::model::evaluation::get_emoji;
use crate::model::game::{Game, GameState};
use crate::model::validate_word::validate_word;
use crate::player::PlayerState;
use crate::util::extract_second_word;
use crate::util::get_regional_indicator;
use crate::wordlist::WordList;

use anyhow::Result;
use serenity::client::Context;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::channel::Message;
use serenity::model::prelude::User;
use serenity::utils::MessageBuilder;
use std::collections::hash_map::Entry;
use std::sync::{Arc, Mutex};

enum GameCreationState {
    AlreadyInProgress,
    SuccessfullyCreated,
    ErrorDuringCreation,
}

fn construct_game_opt_result(
    player_state: &Arc<Mutex<PlayerState>>,
    player_id: u64,
    solution: String,
) -> GameCreationState {
    let mut lock = player_state.lock().unwrap();
    let entry = lock.games_per_player.entry(player_id);
    if matches!(entry, Entry::Occupied(_)) {
        return GameCreationState::AlreadyInProgress;
    }

    match Game::new(solution) {
        Ok(game) => {
            entry.or_insert(game);
            GameCreationState::SuccessfullyCreated
        }
        Err(err) => {
            eprintln!("Error during game creation: {err}");
            GameCreationState::ErrorDuringCreation
        }
    }
}

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
                    let game_creation_state =
                        construct_game_opt_result(player_state, player_id, word);

                    match game_creation_state {
                        GameCreationState::AlreadyInProgress => {
                            msg.reply(ctx, "Game aleady in progress!").await?;
                        }
                        GameCreationState::ErrorDuringCreation => {
                            msg.reply(ctx, format!("Encountered an internal error."))
                                .await?;
                        }
                        GameCreationState::SuccessfullyCreated => {
                            tokio::spawn(game_loop(
                                ctx.clone(),
                                Code { value: code },
                                Arc::clone(word_list),
                                Arc::clone(player_state),
                                msg.author.clone(),
                            ));
                        }
                    }
                }
            }
            Err(_) => {
                msg.reply(ctx, format!("Invalid code: {code}")).await?;
            }
        }
    }
    Ok(())
}

async fn game_loop(
    ctx: Context,
    code: Code,
    word_list: Arc<WordList>,
    player_state: Arc<Mutex<PlayerState>>,
    user: User,
) {
    if let Err(e) = game_loop_logic(ctx, code, word_list, player_state, user).await {
        eprintln!("encountered error in game loop: {e}");
    }
}

async fn game_loop_logic(
    ctx: Context,
    code: Code,
    word_list: Arc<WordList>,
    player_state: Arc<Mutex<PlayerState>>,
    user: User,
) -> Result<()> {
    user.direct_message(&ctx, |m| {
        m.content("You can now start guessing. Good luck.")
    })
    .await?;

    while let Some(msg) = user.await_reply(&ctx).await {
        match {
            let mut lock = player_state.lock().unwrap();
            let game = lock.games_per_player.get_mut(&user.id.0).unwrap();
            let guess = msg.content.to_ascii_lowercase();
            match validate_word(&guess, &word_list.words, game.solution()) {
                Ok(_) => {
                    game.guess(guess, &word_list.words)?;
                    let game = game.clone();
                    if game.state() != GameState::InProgress {
                        // game evaluation is done and the game has been completed -- remove the lock so the player can start the next game
                        // This can be done asyncrhonously while
                        lock.games_per_player.remove(&user.id.0);
                    }
                    Ok(game)
                }
                Err(e) => Err(e),
            }
        } {
            Err(e) => {
                eprintln!("error {e}");
                msg.reply(&ctx, "invalid word!").await?;
            }
            Ok(game) => {
                let game_state = game.state();

                let mut message_builder = MessageBuilder::new();

                let line = match game_state {
                    GameState::Lost => format!("X/{}", constants::MAX_GUESSES),
                    GameState::Won => {
                        format!("{}/{}", game.history().len(), constants::MAX_GUESSES)
                    }
                    GameState::InProgress => format!(
                        "{}/{} [in progress]",
                        game.history().len(),
                        crate::constants::MAX_GUESSES
                    ),
                };

                let code = code.value;
                message_builder.push_line(format!("Friendle {code} {line}"));

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
