mod commands;
use commands::help::*;
use commands::play::*;
use commands::add::*;

use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{macros::group, StandardFramework};
use serenity::model::gateway::Ready;

use std::env;
use std::error::Error;

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

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("."))
        .help(&MY_HELP)
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("WORDLE_DISCORD_TOKEN").expect("token");
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
    Ok(())
}
