//! Tokenizerは入力からTokenを取り出す。
//! TokenはToken構造体。
//! Tokenizerはiteratorをimplementしているため、for文で使える

use std::io::Read;
use std::io::BufReader;
use std::io::SeekFrom;
use std::io::Seek;
use std::iter::Iterator;

mod token;
use token::Token;


pub struct Tokenizer<R> {
    stream: BufReader<R>
}

impl<R: Read + Seek> Tokenizer<R> {
    pub fn new(stream: R) -> Tokenizer<R> {
        Tokenizer {
            stream: BufReader::new(stream)
        }
    }
}

impl<R: Read + Seek> Iterator for Tokenizer<R> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        let mut s = String::new();
        'outer: loop {
            match self.stream.matches(&["/*", "//"]) {
                // ブロックコメントの場合
                Matches::Str("/*") => {
                    // 次の*/まで飛ばす
                    loop {
                        match self.stream.matches(&["*/"]) {
                            // コメントが終わったらloopを抜ける
                            Matches::Str(_) => break,
                            // 関係の無い文字は飛ばす
                            Matches::Char(_) => (), 
                            // コメントのまま終端まで読んだらouterの
                            // ループを抜ける
                            Matches::None => break 'outer
                        }
                    }
                },
                // 行コメントの場合
                Matches::Str("//") => {
                    // 次の\nまで飛ばす
                    loop {
                        match self.stream.matches(&["\r\n", "\n"]) {
                            // \r\nまたは\nが見つかったらloopを抜ける
                            Matches::Str(_) => break,
                            // 関係の無い文字は飛ばす
                            Matches::Char(_) => (),
                            // コメントのまま終端まで読んだらouterの
                            // ループを抜ける
                            Matches::None => break 'outer
                        }
                    }
                },
                Matches::Str(_) => panic!(),
                Matches::Char(c) => match c {
                    // asciiコードの0から32までの場合
                    '\x00'..=' ' => {
                        // すでにトークンがある場合はトークンの分かれ目の
                        // になるので、今までのトークン排出するためbreakする
                        // そうでない場合は何もしない
                        if 0 < s.len() {
                            break;
                        }
                    },
                    // ダブルクオートの場合
                    '"' => {
                        // すでにトークンがある場合は、このトークンを
                        // 読まなかったことにしてすでにあるとトークンを
                        // 排出するためにbreakする
                        // そうでない場合は次のダブルクオートまでの文字を
                        // トークンとして排出する
                        if 0 < s.len() {    
                            let _ = self.stream.seek(SeekFrom::Current(-1));
                            break;
                        }

                        loop {
                            match self.stream.read_char() {
                                // "が見つかったらそれまでの文字列をトークン
                                // として返す
                                Some('"') => {
                                    return Some(Token::String(s))
                                },
                                // 文字はすべて追加する
                                Some(c) => s.push(c),
                                // 終端まで読んでしまった場合はNoneを返す
                                None => return None
                            }
                        }
                    }
                    // シンボルの場合
                    '{' | '}' | '(' | ')' | '[' | ']' | '.' | ',' | ';' | '+' | 
                    '-' | '*' | '/' | '&' | '|' | '<' | '>' | '=' | '~' => {
                        // すでにトークンがある場合は、この文字を
                        // 読まなかったことにしてすでにあるトークンを
                        // 排出するするためにbreakする
                        // そうでない場合はsymbolのトークンとしてreturnする
                        if 0 < s.len() {    
                            let _ = self.stream.seek(SeekFrom::Current(-1));
                            break;
                        } else {
                            return Some(Token::Symbol(c.to_string()))
                        }
                    },
                    // 通常の文字ならsに追加する
                    c => s.push(c),
                },
                // 終端まで読み終えたらbreakする
                Matches::None => break
            }
        }

        if 0 < s.len() {
            // トークンが不明な場合はToken::new()に渡す
            return Some(Token::new(s).unwrap())
        }

        None
    }
}


#[derive(Debug, PartialEq)]
enum Matches<'a> {
    Str(&'a str),
    Char(char),
    None
}

trait MyRead {
    /// ストリームから1byteだけ読み出す
    fn read_byte(&mut self) -> Option<u8>;
    /// ストリームから1文字だけ読み出す
    fn read_char(&mut self) -> Option<char>;
    /// ストリームの先頭から文字を読んで引数の文字列にマッチするものがあれば
    /// それを返す。いずれの文字列にもマッチしなければ先頭の一文字を返す
    fn matches<'a>(&mut self, s_list: &[&'a str]) -> Matches<'a>;
}

impl<R: Read + Seek> MyRead for BufReader<R> {
    /// ストリームから1byteだけ読み出す
    fn read_byte(&mut self) -> Option<u8> {
        let mut c = [0;1];
        if self.read(&mut c).unwrap_or(0) == 0 {
            return None
        }

        Some(c[0])
    }

    /// ストリームから1文字だけ読み出す
    fn read_char(&mut self) -> Option<char> {
        let mut c = [0;1];
        if self.read(&mut c).unwrap_or(0) == 0 {
            return None
        }

        Some(c[0] as char)
    }

    /// ストリームの先頭から文字を読んで引数の文字列にマッチするものがあれば
    /// それを返す。いずれの文字列にもマッチしなければ先頭の一文字を返す
    fn matches<'a>(&mut self, s_list: &[&'a str]) -> Matches<'a> {
        /*
        読み出したbytesより文字列のバイトの方が多ければ、足りない分を読み出す
        リターンするときは読み過ぎた分をシークで戻す
        */
        let mut bytes: Vec<u8> = Vec::new();

        'outer: for s in s_list {
            for (i, b) in s.as_bytes().iter().enumerate() {
                // 足りないときは読み出す
                if bytes.get(i) == None {
                    if let Some(c) = self.read_byte() {
                        bytes.push(c);
                    } else {
                        // 読み出せないときはこのループを終了して次の文字列へ
                        continue 'outer;
                    }
                }

                if bytes[i] != *b {
                    // 一文字でも違うならこのループを終了する
                    continue 'outer;
                }
            }

            // 最後まで一致したらそれを返す
            // しかし、その前に読み過ぎた分をシークで戻す
            let len = bytes.len() - s.len();
            if 0 < len {
                let _ = self.seek(SeekFrom::Current(-(len as i64)));
            }
            return Matches::Str(s)
        }

        // 一文字も読み出していないときはNoneを返す
        // streamから文字を読み出せなかったということ
        if bytes.len() == 0 {
            return Matches::None
        }
        
        // 読みすぎた分を戻して1文字だけ返す
        let len = bytes.len() - 1;
        if 0 < len {
            let _ = self.seek(SeekFrom::Current(-(len as i64)));
        }
        
        return Matches::Char(bytes[0] as char)
    }
}


#[cfg(test)]
mod test {
    use std::io::BufReader;
    use std::io::Cursor;

    use super::Tokenizer;
    use super::token::Token;
    use super::Matches;
    use super::MyRead;

    #[test]
    fn test_tokenizer_next() {
        let mut tokenizer = Tokenizer::new(Cursor::new("aiueo"));
        assert_eq!(tokenizer.next(), Token::new("aiueo".to_string()));
        assert_eq!(tokenizer.next(), None);

        let mut tokenizer = Tokenizer::new(Cursor::new("/*aiueo*/aiueo2"));
        assert_eq!(tokenizer.next(), Token::new("aiueo2".to_string()));

        let mut tokenizer = Tokenizer::new(Cursor::new("aiueo2/*aiueo*/"));
        assert_eq!(tokenizer.next(), Token::new("aiueo2".to_string()));

        let mut tokenizer = Tokenizer::new(Cursor::new("aiueo2//aiueo"));
        assert_eq!(tokenizer.next(), Token::new("aiueo2".to_string()));

        let mut tokenizer = Tokenizer::new(Cursor::new("aiueo*aiueo2"));
        assert_eq!(tokenizer.next(), Token::new("aiueo".to_string()));
        assert_eq!(tokenizer.next(), Token::new("*".to_string()));
        assert_eq!(tokenizer.next(), Token::new("aiueo2".to_string()));

        let mut tokenizer = Tokenizer::new(Cursor::new("aiueo\"aiueo2\"aiueo3"));
        assert_eq!(tokenizer.next(), Token::new("aiueo".to_string()));
        assert_eq!(tokenizer.next(), Token::new("\"aiueo2\"".to_string()));
        assert_eq!(tokenizer.next(), Token::new("aiueo3".to_string()));

        let mut tokenizer = Tokenizer::new(Cursor::new("aiueo\"\"aiueo3"));
        assert_eq!(tokenizer.next(), Token::new("aiueo".to_string()));
        assert_eq!(tokenizer.next(), Token::new("\"\"".to_string()));
        assert_eq!(tokenizer.next(), Token::new("aiueo3".to_string()));

        let mut tokenizer = Tokenizer::new(Cursor::new("aiueo *   aiueo2"));
        assert_eq!(tokenizer.next(), Token::new("aiueo".to_string()));
        assert_eq!(tokenizer.next(), Token::new("*".to_string()));
        assert_eq!(tokenizer.next(), Token::new("aiueo2".to_string()));

        let c = Cursor::new(r#"
        aiueo
        *


        aiueo2
        "#);
        let r = ["aiueo", "*", "aiueo2"];
        assert_eq!(
            Tokenizer::new(c).into_iter().map(|t| t).collect::<Vec<Token>>(),
            r.iter().map(|s| Token::new(s.to_string()).unwrap()).collect::<Vec<Token>>()
        );

        let cursor = Cursor::new(r#"
        aiueo void () {}
        aiueo2 int () {
            return 1 * 2
        }
        "#);
        let r = [
            "aiueo", "void", "(", ")", "{", "}", "aiueo2", "int", "(", ")", 
            "{", "return", "1", "*", "2", "}"
        ];
        assert_eq!(
            Tokenizer::new(cursor).into_iter().map(|t| t).collect::<Vec<Token>>(),
            r.iter().map(|s| Token::new(s.to_string()).unwrap()).collect::<Vec<Token>>()
        );

        let cursor = Cursor::new(r#"
        aiueo void() {
            let c=a/*
            aiueo
            */+b/*aiueo*//*aiueo*/;
            let a; //aiueo
            let b;//aiueo
        }
        "#);
        let r = [
            "aiueo", "void", "(", ")", "{", "let", "c", "=", "a", "+", "b", 
            ";", "let", "a", ";", "let", "b", ";", "}"
        ];
        assert_eq!(
            Tokenizer::new(cursor).into_iter().map(|t| t).collect::<Vec<Token>>(),
            r.iter().map(|s| Token::new(s.to_string()).unwrap()).collect::<Vec<Token>>()
        );
    } 

    #[test]
    fn test_bufreader_matches() {
        let mut bufreader = BufReader::new(Cursor::new(""));
        assert_eq!(bufreader.matches(&["aiueo"]), Matches::None);
        
        let mut bufreader = BufReader::new(Cursor::new("aiueo"));
        assert_eq!(bufreader.matches(&["aiueo"]), Matches::Str("aiueo"));

        let mut bufreader = BufReader::new(Cursor::new("aiueokakikukeko"));
        bufreader.matches(&["aiueo"]);
        assert_eq!(bufreader.matches(&["kakikukeko"]), Matches::Str("kakikukeko"));
        assert_eq!(bufreader.matches(&["a"]), Matches::None);

        let mut bufreader = BufReader::new(Cursor::new("aiueokakikukeko"));
        assert_eq!(bufreader.matches(&["//"]), Matches::Char('a'));
        assert_eq!(bufreader.matches(&["//"]), Matches::Char('i'));

        let mut bufreader = BufReader::new(Cursor::new("aiueo"));
        assert_eq!(bufreader.matches(&["aiueokakikukeko"]), Matches::Char('a'));        
        assert_eq!(bufreader.matches(&["aiueokakikukeko"]), Matches::Char('i'));
        assert_eq!(bufreader.matches(&["aiueokakikukeko"]), Matches::Char('u'));
        assert_eq!(bufreader.matches(&["aiueokakikukeko"]), Matches::Char('e'));
        assert_eq!(bufreader.matches(&["aiueokakikukeko"]), Matches::Char('o')); 
        assert_eq!(bufreader.matches(&["aiueokakikukeko"]), Matches::None);
    }
}