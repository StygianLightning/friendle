use serenity::{
    builder::CreateButton,
    client::Context,
    model::{
        channel::ReactionType,
        interactions::message_component::{ButtonStyle, MessageComponentInteraction},
    },
    utils::MessageBuilder,
};

use crate::{
    model::{
        evaluation::{get_emoji, EmojiMode},
        game::LetterState,
    },
    player::PlayerState,
    util::{adjust_buttons, get_regional_indicator_emoji_with_zero_width_space, KEYBOARD_LAYOUT},
};

pub struct ShowKeyboardButton {}

impl ShowKeyboardButton {
    pub const ID: &'static str = "keyboard";
}

impl ShowKeyboardButton {
    pub fn button() -> CreateButton {
        let mut show_keyboard_button = CreateButton::default();

        show_keyboard_button.custom_id(Self::ID);
        show_keyboard_button.label("show keyboard");
        show_keyboard_button.style(ButtonStyle::Primary);
        show_keyboard_button.emoji(ReactionType::Unicode(String::from("⌨️")));
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
                for row in KEYBOARD_LAYOUT {
                    for c in row.chars() {
                        let state = game.get_letter_state(c);
                        match state {
                            LetterState::Unknown => {
                                msg_builder
                                    .push(get_regional_indicator_emoji_with_zero_width_space(c));
                            }
                            _ => {
                                msg_builder.push(get_emoji(
                                    state.to_evaluation().unwrap(),
                                    EmojiMode::Unicode,
                                ));
                            }
                        }
                    }
                    msg_builder.push_line("");
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
