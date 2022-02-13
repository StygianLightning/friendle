pub const MULTIPLIER: u32 = 30;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Code {
    pub value: u32,
}

pub fn encode(word: &str) -> Code {
    let mut value = 0;
    let mut factor = 1;
    for c in word.chars() {
        value += factor * encode_letter(c);
        factor *= MULTIPLIER;
    }

    Code { value }
}

pub fn decode(code: Code) -> Option<String> {
    let mut s = String::new();
    let mut value = code.value;
    while value > 0 {
        let next_char = char::from_u32('a' as u32 + (value % MULTIPLIER - 1))?;
        value /= MULTIPLIER;
        s += &next_char.to_string();
    }
    Some(s)
}

fn encode_letter(c: char) -> u32 {
    c as u32 - 'a' as u32 + 1
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let word = "abbac";
        assert_eq!(&decode(encode(word)).unwrap(), word);
        let word = "tales";
        assert_eq!(&decode(encode(word)).unwrap(), word);
        let word = "hello";
        assert_eq!(&decode(encode(word)).unwrap(), word);
    }
}
