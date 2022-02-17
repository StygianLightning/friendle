use serenity::{
    builder::CreateButton,
    client::Context,
    model::{
        channel::ReactionType,
        interactions::message_component::{ButtonStyle, MessageComponentInteractionData},
        prelude::User,
    },
};

use crate::player::PlayerState;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ShowKeyboardButton {}

impl ShowKeyboardButton {
    pub const ID: &'static str = "keyboard";
}

impl ShowKeyboardButton {
    pub fn button() -> CreateButton {
        let mut show_keyboard_button = CreateButton::default();

        show_keyboard_button.custom_id(ShowKeyboardButton::ID);
        show_keyboard_button.label("show keyboard");
        show_keyboard_button.style(ButtonStyle::Primary);
        show_keyboard_button.emoji(ReactionType::Unicode(String::from("⌨️")));
        show_keyboard_button
    }

    pub async fn handle_interaction(
        ctx: &Context,
        user: &User,
        data: &MessageComponentInteractionData,
    ) {
        println!("show keyboard button called!");
        let data = ctx.data.read().await;
        let player_state = data.get::<PlayerState>().unwrap();
        let game = {
            let mut lock = player_state.lock().unwrap();
            lock.games_per_player.get_mut(&user.id.0).cloned()
        };
    }
}
