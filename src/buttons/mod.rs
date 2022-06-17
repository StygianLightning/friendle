pub mod copy_result_button;
pub mod show_keyboard_button;

use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

use serenity::{
    client::Context, model::interactions::message_component::MessageComponentInteraction,
};

use copy_result_button::CopyResultButton;
use show_keyboard_button::ShowKeyboardButton;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FriendleButton {
    ShowKeyboard,
    CopyResultButton,
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
            ShowKeyboardButton::ID => Ok(FriendleButton::ShowKeyboard),
            CopyResultButton::ID => Ok(FriendleButton::CopyResultButton),
            _ => Err(ButtonParseError(s.to_string())),
        }
    }
}

impl FriendleButton {
    pub async fn handle_interaction(self, ctx: &Context, mci: &MessageComponentInteraction) {
        if let Err(e) = match self {
            FriendleButton::ShowKeyboard => ShowKeyboardButton::handle_interaction(ctx, mci).await,
            FriendleButton::CopyResultButton => todo!(),
        } {
            eprintln!("Error during button interaction: {e}");
        }
    }
}
