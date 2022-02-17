pub mod show_keyboard_button;

use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

use serenity::{
    client::Context,
    model::{interactions::message_component::MessageComponentInteractionData, prelude::User},
};

use show_keyboard_button::ShowKeyboardButton;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FriendleButton {
    ShowKeyboard(ShowKeyboardButton),
}

#[derive(Debug)]
pub struct ButtonParseError(String);

impl Display for ButtonParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Failed to parse {} as component", self.0)
    }
}

impl StdError for ButtonParseError {}

impl FromStr for FriendleButton {
    type Err = ButtonParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ShowKeyboardButton::ID => Ok(FriendleButton::ShowKeyboard(ShowKeyboardButton {})),
            _ => Err(ButtonParseError(s.to_string())),
        }
    }
}

impl FriendleButton {
    pub async fn handle_interaction(
        self,
        ctx: &Context,
        user: &User,
        data: &MessageComponentInteractionData,
    ) {
        match self {
            FriendleButton::ShowKeyboard(_) => {
                ShowKeyboardButton::handle_interaction(ctx, user, data).await
            }
        }
    }
}
