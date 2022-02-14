pub fn extract_second_word(text: &str) -> Option<&str> {
    text.split_ascii_whitespace().skip(1).take(1).next()
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
}
