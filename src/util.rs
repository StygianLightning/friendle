pub fn extract_second_word(text: &str) -> Option<&str> {
    text.split_ascii_whitespace().skip(1).take(1).next()
}

const REGIONAL_INDICATORS: &[char] = &[
    '🇦', '🇧', '🇨', '🇩', '🇪', '🇫', '🇬', '🇭', '🇮', '🇯', '🇰', '🇱', '🇲', '🇳', '🇴', '🇵', '🇶', '🇷', '🇸',
    '🇹', '🇺', '🇻', '🇼', '🇽', '🇾', '🇿',
];

pub fn get_regional_indicator(letter: char) -> char {
    REGIONAL_INDICATORS[(letter.to_ascii_lowercase() as u32 - 'a' as u32) as usize]
}

pub const KEYBOARD_LAYOUT: &[&str] = &[&"QWERTYUIOP", &"asdfghjkl", &"zxcvbnm"];

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
