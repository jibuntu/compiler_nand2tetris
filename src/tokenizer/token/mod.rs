//! Tokenは１つのtokenに対応する
use std::str::FromStr;

/// Tokenの種類の詳細は233ページに書いてある。Integerは0から32767までの整数。
#[derive(Debug, PartialEq)]
pub enum Token {
    Keyword(Keyword),
    Symbol(char),
    Integer(usize), 
    String(String),
    Identifier(String),
}

/// Token::Keywordの値
#[derive(Debug, PartialEq)]
pub enum Keyword {
    Class,
    Method,
    Function,
    Constructor,
    Int,
    Boolean,
    Char,
    Void,
    Var,
    Static,
    Field,
    Let,
    Do,
    If,
    Else,
    While,
    Return,
    True,
    False,
    Null,
    This
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
            "*" | "/" | "&" | "|" | "<" | ">" | "=" | "~" => {
                Token::Symbol(t.chars().next().unwrap())
            },
            // Keywordの場合
            "class" => Token::Keyword(Keyword::Class),
            "method" => Token::Keyword(Keyword::Method),
            "function" => Token::Keyword(Keyword::Function),
            "constructor" => Token::Keyword(Keyword::Constructor),
            "int" => Token::Keyword(Keyword::Int),
            "boolean" => Token::Keyword(Keyword::Boolean),
            "char" => Token::Keyword(Keyword::Char),
            "void" => Token::Keyword(Keyword::Void),
            "var" => Token::Keyword(Keyword::Var),
            "static" => Token::Keyword(Keyword::Static),
            "field" => Token::Keyword(Keyword::Field),
            "let" => Token::Keyword(Keyword::Let),
            "do" => Token::Keyword(Keyword::Do),
            "if" => Token::Keyword(Keyword::If),
            "else" => Token::Keyword(Keyword::Else),
            "while" => Token::Keyword(Keyword::While),
            "return" => Token::Keyword(Keyword::Return),
            "true" => Token::Keyword(Keyword::True),
            "false" => Token::Keyword(Keyword::False),
            "null" => Token::Keyword(Keyword::Null),
            "this" => Token::Keyword(Keyword::This),
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

    /// テスト用にxmlで書き出す関数
    pub fn to_xml(&self) -> String {
        match self {
            Token::Keyword(t) => format!("<keyword> {} </keyword>", t),
            Token::Symbol(t) if *t == '<' => format!("<symbol> &lt; </symbol>"),
            Token::Symbol(t) if *t == '>' => format!("<symbol> &gt; </symbol>"),
            Token::Symbol(t) if *t == '&' => format!("<symbol> &amp; </symbol>"),
            Token::Symbol(t) => format!("<symbol> {} </symbol>", t),
            Token::Integer(t) => format!("<integerConstant> {} </integerConstant>", t),
            Token::String(t) => format!("<stringConstant> {} </stringConstant>", t),
            Token::Identifier(t) => format!("<identifier> {} </identifier>", t),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Token;

    #[test]
    fn test_token_new() {
        assert_eq!(Token::new("(".to_string()), 
                   Some(Token::Symbol('(')));
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