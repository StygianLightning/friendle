use reqwest::get;
use serde::{Deserialize, Serialize};
use serenity::client::Context;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::channel::Message;

use crate::util::extract_second_word;
use crate::validate_encode_and_post;

#[derive(Deserialize, Serialize, Debug)]
struct Daily {
    id: u32,
    solution: String,
}

#[command]
#[description = "Encode the Wordle solution for a given day. Defaults to the current day in UTC."]
#[only_in(dm)]
pub async fn daily(ctx: &Context, msg: &Message) -> CommandResult {
    // TODO we could cache the daily Wordle solution using context data
    // let data = ctx.data.read().await; // etc

    let date = match extract_second_word(&msg.content) {
        Some(date) => date,
        _ => {
            let today = chrono::Utc::now();
            &format!("{}", today.format("%Y-%m-%d"))
        }
    };

    match get_daily_wordle_solution(date).await {
        Err(e) => {
            eprintln!("{e}");
            if let Err(e2) = msg
                .reply(
                    ctx,
                    format!("could not acquire Wordle solution for `{date}`"),
                )
                .await
            {
                eprintln!("Error posting about problem ({e}) acquiring daily wordle solution: {e2}");
            }
        }

        Ok(daily) => {
            validate_encode_and_post(ctx, msg, &daily.solution).await?;
        }
    }

    Ok(())
}

async fn get_daily_wordle_solution(date: &str) -> anyhow::Result<Daily> {
    let url = format!("https://www.nytimes.com/svc/wordle/v2/{}.json", date);
    let response = get(url).await?.text().await?;
    let daily: Daily = serde_json::from_str(&response)?;
    Ok(daily)
}
