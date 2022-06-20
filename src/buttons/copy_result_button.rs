use serenity::{
    builder::CreateButton,
    client::Context,
    model::{
        channel::ReactionType,
        interactions::message_component::{ButtonStyle, MessageComponentInteraction},
    },
    utils::MessageBuilder,
};

use crate::{player::PlayerState, util::remove_buttons};

pub struct CopyResultButton {}

impl CopyResultButton {
    pub const ID: &'static str = "copy_result";
}

impl CopyResultButton {
    pub fn button() -> CreateButton {
        let mut show_keyboard_button = CreateButton::default();
        show_keyboard_button.custom_id(Self::ID);
        show_keyboard_button.label("Copy Result");
        show_keyboard_button.style(ButtonStyle::Primary);
        show_keyboard_button
    }

    pub async fn handle_interaction(
        ctx: &Context,
        mci: &mut MessageComponentInteraction,
    ) -> anyhow::Result<()> {
        let data = ctx.data.read().await;
        let player_state = data.get::<PlayerState>().unwrap();
        let user = &mci.user;
        let game = {
            let mut lock = player_state.lock().unwrap();
            lock.games_per_player.get_mut(&user.id.0).cloned()
        };

        if game.is_none() {
            return Ok(());
        }
        let game = game.unwrap();

        mci.create_interaction_response(ctx, |r| {
            // todo get evalutions
            r.interaction_response_data(|msg| {
                let mut msg_builder = MessageBuilder::new();
                msg.content(msg_builder.build());
                msg
            });
            r
        })
        .await?;

        remove_buttons(mci, ctx).await?;

        Ok(())
    }
}
