use serenity::{
    builder::CreateButton,
    client::Context,
    model::interactions::message_component::{ButtonStyle, MessageComponentInteraction},
    utils::MessageBuilder,
};

use crate::{
    model::{evaluation::EmojiMode, game::GameState},
    player::PlayerState,
    util::adjust_buttons,
};

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
            r.interaction_response_data(|msg| {
                let mut msg_builder = MessageBuilder::new();
                if game.state() == GameState::InProgress {
                    msg_builder.push_line_safe("The current game isn't finished yet.");
                } else {
                    // Game is finished -> Display state
                    game.display_game_state_header(&mut msg_builder);
                    // evaluation converted to emojis
                    game.display_state(&mut msg_builder, EmojiMode::DiscordName);
                }
                msg.content(msg_builder.build());
                msg
            });
            r
        })
        .await?;

        adjust_buttons(mci, &game, ctx).await?;

        Ok(())
    }
}
