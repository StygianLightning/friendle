use crate::{
    buttons::{mode_button::ModeButton, FriendleButton},
    model::game::Game,
};
use serenity::{
    client::Context,
    model::interactions::message_component::{ActionRowComponent, MessageComponentInteraction},
    prelude::SerenityError,
};
use std::str::FromStr;

pub const KEYBOARD_LAYOUT: &[&str] = &["qwertyuiop", "asdfghjkl", "zxcvbnm"];

const REGIONAL_INDICATORS: &[char] = &[
    '🇦', '🇧', '🇨', '🇩', '🇪', '🇫', '🇬', '🇭', '🇮', '🇯', '🇰', '🇱', '🇲', '🇳', '🇴', '🇵', '🇶', '🇷', '🇸',
    '🇹', '🇺', '🇻', '🇼', '🇽', '🇾', '🇿',
];

fn get_regional_indicator(letter: char) -> char {
    REGIONAL_INDICATORS[(letter.to_ascii_lowercase() as u32 - 'a' as u32) as usize]
}

pub fn get_regional_indicator_emoji_with_zero_width_space(c: char) -> String {
    // add a zero-width space unicode character after each emoji to prevent Serenity from merging successive emojis.
    format!("{}\u{200c}", get_regional_indicator(c))
}

pub fn extract_second_word(text: &str) -> Option<&str> {
    text.split_ascii_whitespace().skip(1).take(1).next()
}

fn match_button_id(button_id: &str, interaction_id: &str, game: &Game) -> Option<FriendleButton> {
    match FriendleButton::from_str(button_id) {
        Ok(button) => {
            if button.id() == interaction_id {
                match button {
                    // clicked buttons are removed, except for mode change buttons, which flip their target mode instead
                    FriendleButton::ModeChangeButton(button) => {
                        let inverted_mode = button.mode.invert();
                        if game.can_switch_to_mode(inverted_mode).is_ok() {
                            Some(FriendleButton::ModeChangeButton(ModeButton {
                                mode: inverted_mode,
                            }))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            } else {
                Some(button)
            }
        }
        Err(e) => {
            eprintln!("Unknown button: {e}");
            None
        }
    }
}

fn get_adjusted_button(
    interaction_id: &str,
    game: &Game,
    component: &ActionRowComponent,
) -> Option<FriendleButton> {
    match component {
        serenity::model::interactions::message_component::ActionRowComponent::Button(button) => {
            button
                .custom_id
                .as_ref()
                .and_then(|button_id| match_button_id(button_id, interaction_id, game))
        }
        serenity::model::interactions::message_component::ActionRowComponent::SelectMenu(_) => {
            None // currently unused; we only use buttons!
        }
        _ => None, // only here for completeness
    }
}

fn collect_adjusted_buttons(mci: &MessageComponentInteraction, game: &Game) -> Vec<FriendleButton> {
    mci.message
        .components
        .first()
        .map(|components| {
            components
                .components
                .iter()
                .filter_map(|component| get_adjusted_button(&mci.data.custom_id, game, component))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub async fn adjust_buttons(
    mci: &mut MessageComponentInteraction,
    game: &Game,
    ctx: &Context,
) -> Result<(), SerenityError> {
    // All our buttons are currently in one action row.
    let buttons = collect_adjusted_buttons(mci, game);
    mci.message
        .edit(ctx, |m| {
            m.components(|c| {
                if !buttons.is_empty() {
                    c.create_action_row(|row| {
                        for button in buttons {
                            row.add_button(button.create_button());
                        }
                        row
                    });
                }
                c
            })
        })
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction() {
        assert!(extract_second_word("text").is_none());
        assert_eq!(extract_second_word(".play tales").unwrap(), "tales");
        assert_eq!(
            extract_second_word(".play tales of arise").unwrap(),
            "tales"
        );
    }

    #[test]
    fn test_regional_indicators() {
        assert_eq!(get_regional_indicator('a'), '🇦');
        assert_eq!(get_regional_indicator('z'), '🇿');
    }

    #[test]
    fn test_all_letters_present_in_layout() {
        let len: usize = KEYBOARD_LAYOUT.iter().map(|s| s.len()).sum();
        assert_eq!(len, 26);
    }
}
