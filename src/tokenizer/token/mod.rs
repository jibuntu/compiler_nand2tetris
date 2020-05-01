//! Token構造体は１つのtokenに対応する

#[derive(Debug, PartialEq)]
pub struct Token {
    token: String,
}

impl Token {
    pub fn new(token: &str) -> Token {
        Token {
            token: token.to_string()
        }
    }
}

#[cfg(test)]
mod test {
    use super::Token;

    #[test]
    fn test_token() {
        assert_eq!(Token::new("test").token, "test".to_string());
    }
}