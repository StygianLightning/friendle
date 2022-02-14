use std::sync::Arc;

use serenity::client::Context;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::channel::Message;
use serenity::model::prelude::User;
use serenity::prelude::RwLock;

use crate::model::coding::{decode, Code};
use crate::model::game::Game;
use crate::player::PlayerState;
use crate::util::extract_second_word;

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

async fn game_loop(ctx: Context, player_state: Arc<RwLock<PlayerState>>, user: User) {
    if let Err(e) = user
        .direct_message(&ctx, |m| {
            m.content("You can now start guessing. Good luck.")
        })
        .await
    {
        eprintln!("Failed to message user {user}: {e}");
        return;
    };

    if let Some(reply) = user.await_reply(&ctx).await {}
}
