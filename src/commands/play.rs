use crate::buttons::mode_button::ModeButton;
use crate::model::coding::{decode, Code};

use crate::model::game::{Game, GameFlag, GameFlags, GameState, StrictMode};

use crate::player::PlayerState;
use crate::util::extract_second_word;
use crate::wordlist::WordList;

use serenity::client::Context;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::channel::Message;
use serenity::utils::MessageBuilder;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

enum GameCreationState {
    AlreadyInProgress,
    SuccessfullyCreated(GameFlags),
    ErrorDuringCreation,
}

fn construct_game_opt_result(
    player_state: &Arc<Mutex<PlayerState>>,
    player_id: u64,
    code: Code,
    solution: String,
    word_list: &HashSet<String>,
) -> GameCreationState {
    let mut lock = player_state.lock().unwrap();
    let game = lock.games_per_player.get(&player_id);

    if let Some(game) = game {
        // Report the game as in-progress only if it has not been finished yet.
        // If the game has already finished, we're free to start a new one.
        if game.state() == GameState::InProgress {
            return GameCreationState::AlreadyInProgress;
        }
    }

    match Game::new(code, solution, word_list) {
        Ok(game) => {
            let flags = game.flags().clone();
            // We could use the entry API above, but Entry::insert_entry is nightly-only, so we'd have to call HashMap::insert anyway.
            lock.games_per_player.insert(player_id, game);
            GameCreationState::SuccessfullyCreated(flags)
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
    // remove back ticks since we escape back ticks when showing the code.
    if let Some(code) = extract_second_word(&msg.content.replace('`', "")) {
        match code.parse::<u32>() {
            Ok(code) => {
                if let Some(word) = decode(Code { value: code }) {
                    let player_id = msg.author.id.0;
                    let data = ctx.data.read().await;
                    let player_state = data.get::<PlayerState>().unwrap();
                    let word_list = data.get::<WordList>().unwrap();
                    let game_creation_state = construct_game_opt_result(
                        player_state,
                        player_id,
                        Code { value: code },
                        word,
                        &word_list.words,
                    );

                    match game_creation_state {
                        GameCreationState::AlreadyInProgress => {
                            msg.reply(ctx, "Game aleady in progress!").await?;
                        }
                        GameCreationState::ErrorDuringCreation => {
                            msg.reply(ctx, String::from("Encountered an internal error."))
                                .await?;
                        }
                        GameCreationState::SuccessfullyCreated(flags) => {
                            let mut msg_builder = MessageBuilder::new();
                            msg_builder.push_line("You can now start guessing. Good luck.");

                            if flags.contains(&GameFlag::SolutionNotInWordList) {
                                msg_builder.push_line("Btw, the solution to this one is not in the original word list.");
                            }

                            msg.channel_id
                                .send_message(ctx, |m| {
                                    m.content(msg_builder);
                                    m.components(|comps| {
                                        comps.create_action_row(|row| {
                                            row.add_button(
                                                ModeButton::new(StrictMode::Enabled).mode_button(),
                                            );
                                            row
                                        });
                                        comps
                                    });
                                    m
                                })
                                .await?;
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
