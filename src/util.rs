pub fn extract_second_word(text: &str) -> Option<&str> {
    text.split_ascii_whitespace().skip(1).take(1).next()
}

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

pub const KEYBOARD_LAYOUT: &[&str] = &[&"qwertyuiop", &"asdfghjkl", &"zxcvbnm"];

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
        let len = KEYBOARD_LAYOUT
            .iter()
            .map(|s| s.len())
            .fold(0, |a, b| a + b);
        assert_eq!(len, 26);
    }
}
