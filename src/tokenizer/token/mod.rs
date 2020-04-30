//! Token構造体は１つのtokenに対応する

pub struct Token {
    token: String,
}

impl Token {
    pub fn new(token: String) -> Token {
        Token {
            token
        }
    }
}