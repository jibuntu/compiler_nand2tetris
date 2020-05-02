//! Tokenは１つのtokenに対応する
use std::str::FromStr;

/// Tokenの種類の詳細は233ページに書いてある。Integerは0から32767までの整数。
#[derive(Debug, PartialEq)]
pub enum Token {
    Keyword(String),
    Symbol(String),
    Integer(usize), 
    String(String),
    Identifier(String),
}

impl Token {
    /// 引数の文字列を元に適切なTokenを返す。無効なtokenの場合はNoneを返す
    pub fn new(t: String) -> Option<Token> {
        if t.len() == 0 {
            return None
        }

        let token = match t.as_str() {
            // Symbolの場合
            "{" | "}" | "(" | ")" | "[" | "]" | "." | "," | ";" | "+" | "-" | 
            "*" | "/" | "&" | "|" | "<" | ">" | "=" | "~" => Token::Symbol(t),
            // Keywordの場合
            "class" | "constructor" | "function" | "method" | "field" | 
            "static" | "var" | "int" | "char" | "boolean" | "void" | "true" |
            "false" | "null" | "this" | "let" | "do" | "if" | "else" | 
            "while" | "return" => Token::Keyword(t),
            _ => match usize::from_str(&t) {
                Ok(u) => Token::Integer(u),
                _ => {
                    let mut chars = t.chars();
                    match (chars.next(), chars.last()) {
                        // 最初と最後が"の場合は文字列とする
                        (Some(c), Some(l)) if c == '"' && l == '"' => {
                            if &t == "\"" {
                                // "だけだった場合はNoneを返す
                                return None
                            }
                            Token::String(
                                t.get(1..t.len()-1).unwrap().to_string())
                        },
                        // いずれにも当てはまらないときはIdentifierとする
                        _ => Token::Identifier(t)
                    }
                }
            }
        };

        Some(token)
    }
}

#[cfg(test)]
mod test {
    use super::Token;

    #[test]
    fn test_token_new() {
        assert_eq!(Token::new("(".to_string()), 
                   Some(Token::Symbol("(".to_string())));
        assert_eq!(Token::new("class".to_string()), 
                   Some(Token::Keyword("class".to_string())));
        assert_eq!(Token::new("0".to_string()), 
                   Some(Token::Integer(0)));
        assert_eq!(Token::new("20".to_string()), 
                   Some(Token::Integer(20)));
        assert_eq!(Token::new("\"aiueo\"".to_string()), 
                   Some(Token::String("aiueo".to_string())));
        assert_eq!(Token::new("\"\"".to_string()), 
                   Some(Token::String("".to_string())));
        assert_eq!(Token::new("aiueo".to_string()),
                   Some(Token::Identifier("aiueo".to_string())));
    }
}