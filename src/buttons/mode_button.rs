use serenity::{
    builder::CreateButton,
    client::Context,
    model::interactions::message_component::{ButtonStyle, MessageComponentInteraction},
    utils::MessageBuilder,
};

use crate::{
    model::game::{ModeChangeError, StrictMode},
    player::PlayerState,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ModeButton {
    pub mode: StrictMode,
}

pub const STRICT_MODE_BUTTON_ID: &str = "strict";

pub const NON_STRICT_MODE_BUTTON_ID: &str = "nonstrict";

impl ModeButton {
    pub fn get_id(self) -> &'static str {
        match self.mode {
            StrictMode::Disabled => NON_STRICT_MODE_BUTTON_ID,
            StrictMode::Enabled => STRICT_MODE_BUTTON_ID,
        }
    }

    pub fn new(mode: StrictMode) -> Self {
        Self { mode }
    }
}

impl ModeButton {
    pub fn mode_button(self) -> CreateButton {
        let mut show_keyboard_button = CreateButton::default();
        show_keyboard_button.custom_id(self.get_id());
        let mode_text = match self.mode {
            StrictMode::Enabled => "Enable strict mode",
            StrictMode::Disabled => "Disable strict mode",
        };
        show_keyboard_button.label(mode_text);
        show_keyboard_button.style(ButtonStyle::Primary);
        show_keyboard_button
    }

    pub async fn handle_interaction(
        self,
        ctx: &Context,
        mci: &MessageComponentInteraction,
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
        let mut game = game.unwrap();

        println!("{:?}", game);

        let change_message = match game.set_strict_mode(self.mode) {
            Err(ModeChangeError::AlreadySet) => "Requested mode is already set.",
            Err(ModeChangeError::TooManyGuessesAlready) => {
                "Cannot switch to strict mode with more than one guess."
            }
            Ok(()) if game.get_strict_mode() == StrictMode::Disabled => "Disabled strict mode.",
            Ok(()) => "Enabled strict mode.",
        };

        {
            // Save the updated game state.
            let mut lock = player_state.lock().unwrap();
            lock.games_per_player.insert(user.id.0, game);
        }

        mci.create_interaction_response(ctx, |r| {
            r.interaction_response_data(|msg| {
                let mut msg_builder = MessageBuilder::new();
                msg_builder.push_line_safe(change_message);
                msg.content(msg_builder.build());
                msg
            });
            r
        })
        .await?;

        Ok(())
    }
}
