use std::io::{Read, Write, Seek};

use super::tokenizer::Tokenizer;
use super::tokenizer::token::Token;
use super::tokenizer::token::Keyword;


macro_rules! ErrUnexpect {
    ($self:expr, $expect:expr) => {
        Err(format!("unexpected token: '{}' at line {}. 予期されるトークンは {} です。", 
                    $self.tokenizer.get_current_token().unwrap().to_string(),
                    $self.tokenizer.get_line_number(),
                    $expect))
    };
}

macro_rules! MatchType {
    ($token:expr, $line_number:expr) => {
        match $token {
            Some(t) => match t {
                Token::Keyword(Keyword::Int) | Token::Keyword(Keyword::Char) |
                Token::Keyword(Keyword::Boolean) |
                Token::Identifier(_) => t,
                _ => return Err(format!("unexpected token: '{}' at line {}. 予期されるトークンは 'int', 'char', 'boolean' or className です。",
                                        t.to_string(),
                                        $line_number))
            },
            _ => return Err("type がありません".to_string())
        }
    }
}

macro_rules! MatchIdentifier {
    ($token:expr, $line_number:expr, $expect:expr) => {
        match $token {
            Some(t) => match t {
                Token::Identifier(_) => t,
                _ => return Err(format!("unexpected token: '{}' at line {}. 予期されるトークンは {} です。",
                                        t.to_string(),
                                        $line_number,
                                        $expect))
            },
            _ => return Err("identifier がありません".to_string())
        }
    }
}

pub struct CompilationEngine<R, W> {
    pub tokenizer: Tokenizer<R>,
    output: W,
}

impl<R: Read + Seek, W: Write> CompilationEngine<R, W> {
    pub fn new(tokenizer: Tokenizer<R>, output: W) -> CompilationEngine<R, W> {
        CompilationEngine {
            tokenizer,
            output
        }
    }

    /// tokenizerからクラスをコンパイルし、結果を書き込む。
    /// 最初はvmコードではなくxmlの構文木を書き書き込む。
    pub fn compile_class(&mut self) -> Result<(), String> {
        let _ = self.output.write(b"<class>\n");
        let t = match self.tokenizer.get_current_token() {
            Some(t) => match t {
                Token::Keyword(Keyword::Class) => t,
                _ => return ErrUnexpect!(self, "'class'")
            },
            _ => return Err("class がありません".to_string())
        };
        let _ = self.output.write(format!("{}\n", t.to_xml()).as_bytes());

        let t = match self.tokenizer.advance() {
            Some(t) => match t {
                Token::Identifier(_) => t,
                _ => return ErrUnexpect!(self, "className")
            },
            _ => return Err("クラス名がありません".to_string())
        };
        let _ = self.output.write(format!("{}\n", t.to_xml()).as_bytes());

        let t = match self.tokenizer.advance() {
            Some(t) => match t {
                Token::Symbol('{') => t,
                _ => return ErrUnexpect!(self, "'{'"),
            },
            _ => return Err("'{' トークンがありません".to_string())
        };
        let _ = self.output.write(format!("{}\n", t.to_xml()).as_bytes());

        // classVarDecもしくはsubroutineDec、'}'
        while let Some(t) = self.tokenizer.advance() { 
            match t {
                // classVarDecの場合 
                Token::Keyword(Keyword::Static) | 
                Token::Keyword(Keyword::Field) => {
                    self.class_var_dec()?;
                },
                // subroutineDecの場合
                Token::Keyword(Keyword::Constructor) |
                Token::Keyword(Keyword::Function) | 
                Token::Keyword(Keyword::Method) => {
                    self.subroutine_dec()?;
                },
                // '}'まで読み終えたらOk(())を返す
                Token::Symbol('}') => {
                    let _ = self.output.write(t.to_xml().as_bytes());
                    let _ = self.output.write(b"\n");
                    let _ = self.output.write(b"</class>\n");
                    return Ok(())
                },
                // それ以外はエラーになる
                _ => {
                    return ErrUnexpect!(self, "'static', 'field', 'constructor', 'function', 'method' or '}'")
                }
            }
        }

        // '}'を見つけずに最後まで読み終えたら
        Err("'}' トークンがありません".to_string())
    }
    
    fn class_var_dec(&mut self) -> Result<(), String> {
        let _ = self.output.write(b"<classVarDec>\n");

        match self.tokenizer.get_current_token() {
            Some(t) => match t {
                Token::Keyword(Keyword::Static) |
                Token::Keyword(Keyword::Field) => {
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());
                },
                _ => return ErrUnexpect!(self, "'static' or 'field'")
            },
            None => return Err("'staitc' or 'field' がありません".to_string())
        }

        let t = MatchType!(self.tokenizer.advance(), 
                           self.tokenizer.get_line_number());
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

        let t = MatchIdentifier!(self.tokenizer.advance(), 
                                 self.tokenizer.get_line_number(),
                                 "変数名");
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

        while let Some(t) = self.tokenizer.advance() {
            // 次に';'が来たらreturn、','が来たら繰り返す。それ以外ならエラーを
            // 返す
            match t {
                Token::Symbol(';') => {
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());
                    let _ = self.output.write(b"</classVarDec>\n");
                    return Ok(())
                },
                Token::Symbol(',') => {
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());
                },
                _ => return ErrUnexpect!(self, "予期されるトークンは ';' or ',' です。")
            }

            let t = MatchIdentifier!(self.tokenizer.advance(), 
                                     self.tokenizer.get_line_number());
            let _ = self.output.write((t.to_xml() + "\n").as_bytes());
        }

        Err("';' がありません".to_string())
    }

    fn subroutine_dec(&self) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use super::CompilationEngine;
    use super::Tokenizer;

    #[test]
    fn test_compilation_engine_compile_class() {
        let t = Tokenizer::new(Cursor::new("class test{}"));
        let mut c = CompilationEngine::new(t, Cursor::new(Vec::new()));
        c.tokenizer.advance();
        assert_eq!(c.compile_class(), Ok(()));

        let s: String = c.output.get_ref().iter().map(|b|*b as char).collect();
        assert_eq!(&s.replace(" ","").replace("\n",""), 
                   "<class><keyword>class</keyword><identifier>test</identifier><symbol>{</symbol><symbol>}</symbol></class>");

        let t = Tokenizer::new(Cursor::new("test{"));
        let mut c = CompilationEngine::new(t, Cursor::new(Vec::new()));
        assert_ne!(c.compile_class(), Ok(()));
    }
}