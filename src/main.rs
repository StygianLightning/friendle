mod commands;
mod constants;
mod model;
mod wordlist;

use crate::model::validate_word::validate_word_format;
use crate::wordlist::WordList;
use commands::add::*;
use commands::help::*;
use commands::play::*;
use serenity::prelude::RwLock;
use std::collections::HashSet;

use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{macros::group, StandardFramework};
use serenity::model::gateway::Ready;

use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

#[group]
#[commands(add, play)]
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
    let dict_path = std::env::var(WORD_LIST_PATH_ENV_VAR)
        .unwrap_or_else(|_| String::from("resources/wordlist.txt"));
    println!("word list path env var: {}", dict_path);

    let dict_path = PathBuf::from(dict_path);

    let words_string = std::fs::read_to_string(&dict_path).expect("Failed to load word list");
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
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var(DISCORD_TOKEN).expect("token");
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    let dict = Arc::new(RwLock::new(word_list));
    {
        let mut data = client.data.write().await;
        data.insert::<WordList>(Arc::clone(&dict));
    }

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
    Ok(())
}
