use std::str::FromStr;

use serenity::async_trait;
use serenity::client::{Context, EventHandler};

use serenity::model::gateway::Ready;
use serenity::model::interactions::message_component::ComponentType;


use serenity::model::interactions::Interaction;

use crate::buttons::FriendleButton;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::MessageComponent(ref mci)
                if mci.data.component_type == ComponentType::Button =>
            {
                let custom_id = &mci.data.custom_id;
                match FriendleButton::from_str(custom_id) {
                    Ok(button) => {
                        button.handle_interaction(&ctx, mci).await;
                    }

                    Err(err) => {
                        eprintln!("button interaction {mci:?} led to error {err}");
                        return;
                    }
                }
            }
            _ => {
                println!("non-button interaction: {interaction:?}");
            }
        }
    }
}
