mod commands;
mod constants;
mod game_loop_hook;
mod model;
mod player;
mod util;
mod wordlist;

use crate::game_loop_hook::message_hook;
use crate::model::validate_word::validate_word_format;
use crate::player::PlayerState;
use crate::wordlist::WordList;
use commands::encode::*;
use commands::help::*;
use commands::play::*;
use std::collections::HashSet;
use std::sync::Mutex;

use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{macros::group, StandardFramework};
use serenity::model::gateway::Ready;

use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

#[group]
#[commands(encode, play)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

pub const WORD_LIST_PATH_ENV_VAR: &str = "WORD_LIST_PATH";
pub const DISCORD_TOKEN: &str = "FRIENDLE_DISCORD_TOKEN";

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let word_list_path = std::env::var(WORD_LIST_PATH_ENV_VAR)
        .unwrap_or_else(|_| String::from("resources/wordlist.txt"));
    println!("word list path env var: {}", word_list_path);

    let word_list_path = PathBuf::from(word_list_path);

    let words_string = std::fs::read_to_string(&word_list_path).expect("Failed to load word list");
    let word_lines = words_string.lines();
    let mut words: HashSet<String> = HashSet::new();
    for word in word_lines {
        match validate_word_format(word) {
            Ok(_) => {
                words.insert(String::from(word));
            }
            Err(err) => {
                eprintln!("{err}");
            }
        }
    }
    let word_list = WordList { words };

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("."))
        .help(&MY_HELP)
        .normal_message(message_hook)
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var(DISCORD_TOKEN).expect("token");
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<WordList>(Arc::new(word_list));
        data.insert::<PlayerState>(Arc::new(Mutex::new(PlayerState::default())));
    }

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
    Ok(())
}
