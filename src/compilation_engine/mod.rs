use std::io::{Read, Write, Seek};

use super::tokenizer::Tokenizer;
use super::tokenizer::token::Token;
use super::tokenizer::token::Keyword;


macro_rules! ErrUnexpect {
    ($self:expr) => {
        Err(format!("unexpected token: '{}' at line {}", 
                    $self.tokenizer.get_current_token().unwrap().to_string(),
                    $self.tokenizer.get_line_number()))
    };
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
                _ => return ErrUnexpect!(self)
            },
            _ => return Err("class がありません".to_string())
        };
        let _ = self.output.write(format!("{}\n", t.to_xml()).as_bytes());

        let t = match self.tokenizer.advance() {
            Some(t) => match t {
                Token::Identifier(_) => t,
                _ => return ErrUnexpect!(self)
            },
            _ => return Err("クラス名がありません".to_string())
        };
        let _ = self.output.write(format!("{}\n", t.to_xml()).as_bytes());

        let t = match self.tokenizer.advance() {
            Some(t) => match t {
                Token::Symbol('{') => t,
                _ => return ErrUnexpect!(self),
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
                    return ErrUnexpect!(self)
                }
            }
        }

        // '}'を見つけずに最後まで読み終えたら
        Err("'}' トークンがありません".to_string())
    }
    
    fn class_var_dec(&self) -> Result<(), String> {
        Ok(())
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