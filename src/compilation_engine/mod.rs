 std::io::{Read, Write, Seek};

use super::tokenizer::Tokenizer;
use super::tokenizer::token::Token;
use super::tokenizer::token::Keyword;


macro_rules! ErrUnexpect {
    ($t:expr, $n:expr) => {
        Err(format!("unexpected token: '{}' at line {}", $t, $n))
    };
}

pub struct CompilationEngine<R, W> {
    tokenizer: Tokenizer<R>,
    output: W,
}

impl<R: Read + Seek, W: Write> CompilationEngine<R, W> {
    pub fn new(tokenizer: Tokenizer<R>, output: W) -> CompilationEngine<R, W> {
        CompilationEngine {
            tokenizer,
            output
        }
    }

    pub fn compile_file(&mut self) -> Result<(), String> {
        match self.tokenizer.next() {
            Some(t) => match t {
                Token::Keyword(Keyword::Class) => {
                    let _ = self.output.write(b"<class>\n");
                    let _ = self.output.write(t.to_xml().as_bytes());
                    let _ = self.output.write(b"\n");
                    self.compile_class()?;
                    let _ = self.output.write(b"</class>\n");
                },
                _ => {
                    return ErrUnexpect!(t.to_string(),
                                        self.tokenizer.get_line_number())
                }
            },
            None => ()
        }

        Ok(())
    }

    /// tokenizerからクラスをコンパイルし、結果を書き込む。
    /// 最初はvmコードではなくxmlの構文木を書き書き込む。
    fn compile_class(&mut self) -> Result<(), String> {
        match self.tokenizer.next() {
            Some(Token::Identifier(t)) => {
                let _ = self.output.write(
                    format!("{}\n", Token::Identifier(t).to_xml()).as_bytes());
            },
            _ => return Err("クラス名がありません".to_string())
        }

        match self.tokenizer.next() {
            Some(t) if t == Token::Symbol('{') => {
                let _ = self.output.write(t.to_xml().as_bytes());
                let _ = self.output.write(b"\n");
            },
            _ => return Err("'{' トークンがありません".to_string())
        }

        // classVarDecもしくはsubroutineDec、'}'
        while let Some(t) = self.tokenizer.next() { 
            match t {
                // classVarDecの場合 
                Token::Keyword(Keyword::Static) | 
                Token::Keyword(Keyword::Field) => {
                    let _ = self.output.write(b"<classVarDec>\n");
                    let _ = self.output.write(t.to_xml().as_bytes());
                    let _ = self.output.write(b"\n");
                    self.class_var_dec()?;
                    let _ = self.output.write(b"</classVarDec>\n");
                },
                // subroutineDecの場合
                Token::Keyword(Keyword::Constructor) |
                Token::Keyword(Keyword::Function) | 
                Token::Keyword(Keyword::Method) => {
                    let _ = self.output.write(b"<subroutineDec>\n");
                    let _ = self.output.write(t.to_xml().as_bytes());
                    let _ = self.output.write(b"\n");
                    self.subroutine_dec()?;
                    let _ = self.output.write(b"</subroutineDec>\n");
                },
                // '}'まで読み終えたらOk(())を返す
                Token::Symbol('}') => {
                    let _ = self.output.write(t.to_xml().as_bytes());
                    let _ = self.output.write(b"\n");
                    return Ok(())
                },
                // それ以外はエラーになる
                _ => {
                    return ErrUnexpect!(t.to_string(),
                                        self.tokenizer.get_line_number())
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
    fn test_compilation_engine() {
        let t = Tokenizer::new(Cursor::new("test{}"));
        let mut c = CompilationEngine::new(t, Cursor::new(Vec::new()));
        assert_eq!(c.compile_class(), Ok(()));

        let s: String = c.output.get_ref().iter().map(|b|*b as char).collect();
        assert_eq!(&s.replace(" ","").replace("\n",""), 
                   "<identifier>test</identifier><symbol>{</symbol><symbol>}</symbol>");

        let t = Tokenizer::new(Cursor::new("test{"));
        let mut c = CompilationEngine::new(t, Cursor::new(Vec::new()));
        assert_ne!(c.compile_class(), Ok(()));
    }
}