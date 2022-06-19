pub mod copy_result_button;
pub mod mode_button;
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

use crate::model::game::StrictMode;
use mode_button::ModeButton;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FriendleButton {
    ShowKeyboard,
    ModeChangeButton(ModeButton),
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
            mode_button::STRICT_MODE_BUTTON_ID => {
                Ok(FriendleButton::ModeChangeButton(ModeButton {
                    mode: StrictMode::Enabled,
                }))
            }
            mode_button::NON_STRICT_MODE_BUTTON_ID => {
                Ok(FriendleButton::ModeChangeButton(ModeButton {
                    mode: StrictMode::Disabled,
                }))
            }
            CopyResultButton::ID => Ok(FriendleButton::CopyResultButton),
            _ => Err(ButtonParseError(s.to_string())),
        }
    }
}

impl FriendleButton {
    pub async fn handle_interaction(self, ctx: &Context, mci: &mut MessageComponentInteraction) {
        if let Err(e) = match self {
            FriendleButton::ShowKeyboard => ShowKeyboardButton::handle_interaction(ctx, mci).await,
            FriendleButton::ModeChangeButton(mode) => mode.handle_interaction(ctx, mci).await,
            FriendleButton::CopyResultButton => {
                CopyResultButton::handle_interaction(ctx, mci).await
            }
        } {
            eprintln!("Error during button interaction: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::buttons::mode_button::{NON_STRICT_MODE_BUTTON_ID, STRICT_MODE_BUTTON_ID};

    use super::*;

    #[test]
    fn button_id() {
        assert!(matches!(
            FriendleButton::from_str(ShowKeyboardButton::ID),
            Ok(FriendleButton::ShowKeyboard)
        ));
        assert!(matches!(
            FriendleButton::from_str(CopyResultButton::ID),
            Ok(FriendleButton::CopyResultButton)
        ));
        assert!(matches!(
            FriendleButton::from_str(NON_STRICT_MODE_BUTTON_ID),
            Ok(FriendleButton::ModeChangeButton(ModeButton {
                mode: StrictMode::Disabled
            }))
        ));
        assert!(matches!(
            FriendleButton::from_str(STRICT_MODE_BUTTON_ID),
            Ok(FriendleButton::ModeChangeButton(ModeButton {
                mode: StrictMode::Enabled
            }))
        ));
    }
}
