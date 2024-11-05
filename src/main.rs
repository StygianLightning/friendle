mod buttons;
mod commands;
mod constants;
mod event_handler;
mod game_loop_hook;
mod model;
mod player;
mod util;
mod wordlist;

use commands::daily::*;
use commands::encode::*;
use commands::help::*;
use commands::play::*;

use event_handler::Handler;
use game_loop_hook::message_hook;
use model::validate_word::validate_word_format;
use player::PlayerState;
use wordlist::WordList;

use serenity::client::Client;
use serenity::framework::standard::{macros::group, StandardFramework};

use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

#[group]
#[commands(encode, play, daily)]
struct General;

pub const WORD_LIST_PATH_ENV_VAR: &str = "WORD_LIST_PATH";
pub const DISCORD_TOKEN: &str = "FRIENDLE_DISCORD_TOKEN";
pub const DISCORD_APPLICATION_ID: &str = "FRIENDLE_APPLICATION_ID";

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    // The Application Id is usually the Bot User Id. It is needed for components
    let application_id: u64 = env::var(DISCORD_APPLICATION_ID)
        .unwrap_or_else(|_| panic!("{DISCORD_APPLICATION_ID} not set"))
        .parse()
        .expect("application id is not a valid id");

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
        .application_id(application_id)
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
