mod commands;
mod wordlist;

use crate::wordlist::WordList;
use commands::add::*;
use commands::help::*;
use commands::play::*;
use serenity::prelude::RwLock;

use anyhow::anyhow;
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

pub const DICT_PATH_ENV_VAR: &str = "DICT_PATH";
pub const WORDLE_DISCORD_TOKEN: &str = "WORDLE_DISCORD_TOKEN";

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let dict_path =
        std::env::var(DICT_PATH_ENV_VAR).unwrap_or_else(|_| String::from("dictionary.json"));
    println!("channel links env var: {}", dict_path);

    if !dict_path.ends_with(".json") {
        anyhow!("Channel links env var is not pointing to a json file!");
    }

    let dict_path = PathBuf::from(dict_path);

    let dict: WordList = std::fs::read_to_string(&dict_path)
        .map(|json| {
            serde_json::from_str::<WordList>(&json).expect("Failed to load saved channel links")
        })
        .unwrap_or_else(|_| Default::default());

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("."))
        .help(&MY_HELP)
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var(WORDLE_DISCORD_TOKEN).expect("token");
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    let dict = Arc::new(RwLock::new(dict));
    {
        let mut data = client.data.write().await;
        data.insert::<WordList>(Arc::clone(&dict));
    }

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
    Ok(())
}
