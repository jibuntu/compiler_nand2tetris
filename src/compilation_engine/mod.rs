use std::io::{Read, Write, Seek};

use super::tokenizer::Tokenizer;
use super::tokenizer::token::Token;
use super::tokenizer::token::Keyword;


macro_rules! ErrReachedEnd {
    () => {
        Err("reached end of file while parsing".to_string())
    };
}

macro_rules! ErrUnexpect {
    ($token:expr, $line_number:expr) => {
        Err(format!("unexpected token: '{}' at line {}.", 
                    $token.to_string(),
                    $line_number))
    };
}

// 引数のトークンにマッチしたらそのトークンを返す。
// そうでなければエラーをreturnする
macro_rules! MatchToken {
    ($token:expr, $line_number:expr, $( $expected_token:pat ),*) => {
        match $token {
            Some(t) => match t {
                $(
                    $expected_token => t,
                )*
                // 予期しないトークンのときはエラーを返す
                _ => return ErrUnexpect!(t, $line_number)
            },
            // tokenがNoneになるのは最後まで読み終えたときなのでエラーを返す
            // javacのエラー文を流用
            None => return ErrReachedEnd!()
        }
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

        let t = MatchToken!(self.tokenizer.get_current_token(),
                            self.tokenizer.get_line_number(),
                            Token::Keyword(Keyword::Class));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

        let t = MatchToken!(self.tokenizer.advance(),
                            self.tokenizer.get_line_number(),
                            Token::Identifier(_));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

        let t = MatchToken!(self.tokenizer.advance(),
                            self.tokenizer.get_line_number(),
                            Token::Symbol('{'));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

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
                    self.compile_subroutine()?;
                },
                // '}'まで読み終えたらOk(())を返す
                Token::Symbol('}') => {
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());
                    let _ = self.output.write(b"</class>\n");
                    return Ok(())
                },
                // それ以外はエラーになる
                _ => return ErrUnexpect!(t, self.tokenizer.get_line_number())
            }
        }

        // '}'を見つけずに最後まで読み終えたらエラーを返す
        ErrReachedEnd!()
    }
    
    fn class_var_dec(&mut self) -> Result<(), String> {
        let _ = self.output.write(b"<classVarDec>\n");

        let t = MatchToken!(self.tokenizer.get_current_token(),
                            self.tokenizer.get_line_number(),
                            Token::Keyword(Keyword::Static),
                            Token::Keyword(Keyword::Field));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

        let t = MatchToken!(self.tokenizer.advance(),
                            self.tokenizer.get_line_number(),
                            Token::Keyword(Keyword::Int),
                            Token::Keyword(Keyword::Char),
                            Token::Keyword(Keyword::Boolean),
                            Token::Identifier(_));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

        let t = MatchToken!(self.tokenizer.advance(),
                            self.tokenizer.get_line_number(),
                            Token::Identifier(_));
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
                _ => return ErrUnexpect!(t, self.tokenizer.get_line_number())
            }

            let t = MatchToken!(self.tokenizer.advance(),
                                self.tokenizer.get_line_number(),
                                Token::Identifier(_));
            let _ = self.output.write((t.to_xml() + "\n").as_bytes());
        }

        ErrReachedEnd!()
    }

    fn compile_subroutine(&mut self) -> Result<(), String> {
        let _ = self.output.write(b"<subroutineDec>\n");

        let t = MatchToken!(self.tokenizer.get_current_token(),
                            self.tokenizer.get_line_number(),
                            Token::Keyword(Keyword::Constructor),
                            Token::Keyword(Keyword::Function),
                            Token::Keyword(Keyword::Method));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

        let t = MatchToken!(self.tokenizer.advance(),
                            self.tokenizer.get_line_number(),
                            Token::Keyword(Keyword::Void),
                            Token::Keyword(Keyword::Int),
                            Token::Keyword(Keyword::Char),
                            Token::Keyword(Keyword::Boolean),
                            Token::Identifier(_));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

        let t = MatchToken!(self.tokenizer.advance(),
                            self.tokenizer.get_line_number(),
                            Token::Identifier(_));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

        let t = MatchToken!(self.tokenizer.advance(),
                            self.tokenizer.get_line_number(),
                            Token::Symbol('('));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

        // 閉じ括弧でなければcompile_parameter_listを実行する
        if self.tokenizer.advance() != Some(&Token::Symbol(')')) {
            self.compile_parameter_list()?;
        }

        // compile_parameter_listは１つ先読みしているので、
        // ここではadvanceを呼ばずに現在のトークンを使う
        let t = MatchToken!(self.tokenizer.get_current_token(),
                            self.tokenizer.get_line_number(),
                            Token::Symbol(')'));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

        let _ = self.output.write(b"<subroutineBody>\n");

        let t = MatchToken!(self.tokenizer.advance(),
                            self.tokenizer.get_line_number(),
                            Token::Symbol('{'));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

        while let Some(t) = self.tokenizer.advance() {
            match t {
                Token::Keyword(Keyword::Var) => {
                    self.compile_var_dec()?;
                },
                // varじゃなければbreakする
                _ => break
            }
        }

        self.compile_statements()?;

        // compile_statementsは１つ先読みしているので、
        // ここではadvanceを呼ばずに現在のトークンを使う
        let t = MatchToken!(self.tokenizer.get_current_token(),
                            self.tokenizer.get_line_number(),
                            Token::Symbol('}'));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

        let _ = self.output.write(b"</subroutineBody>\n");
        let _ = self.output.write(b"</subroutineDec>\n");

        Ok(())
    }

    fn compile_parameter_list(&mut self) -> Result<(), String> {
        let _ = self.output.write(b"<parameterList>\n");

        loop {
            let t = match self.tokenizer.get_current_token() {
                Some(t) => match t {
                    Token::Keyword(Keyword::Int) | 
                    Token::Keyword(Keyword::Char) |
                    Token::Keyword(Keyword::Boolean) |
                    Token::Identifier(_) => t,
                    // typeでなければbrearする
                    _ => break
                },
                None => return ErrReachedEnd!()
            };
            let _ = self.output.write((t.to_xml() + "\n").as_bytes());


            let t = MatchToken!(self.tokenizer.advance(),
                                self.tokenizer.get_line_number(),
                                Token::Identifier(_));
            let _ = self.output.write((t.to_xml() + "\n").as_bytes());

            let t = match self.tokenizer.advance() {
                Some(t) => match t {
                    Token::Symbol(',') => t,
                    // それ以外の文字はbreakする
                    _ => break
                },
                None => return ErrReachedEnd!()
            };

            let _ = self.output.write((t.to_xml() + "\n").as_bytes());
            self.tokenizer.advance();
        }

        let _ = self.output.write(b"</parameterList>\n");
        Ok(())
    }

    fn compile_var_dec(&mut self) -> Result<(), String> {
        let _ = self.output.write(b"<varDec>\n");

        let t = MatchToken!(self.tokenizer.get_current_token(),
                            self.tokenizer.get_line_number(),
                            Token::Keyword(Keyword::Var));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

        let t = MatchToken!(self.tokenizer.advance(),
                            self.tokenizer.get_line_number(),
                            Token::Keyword(Keyword::Int),
                            Token::Keyword(Keyword::Char),
                            Token::Keyword(Keyword::Boolean),
                            Token::Identifier(_));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

        let t = MatchToken!(self.tokenizer.advance(),
                            self.tokenizer.get_line_number(),
                            Token::Identifier(_));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

        loop {
            match self.tokenizer.advance() {
                Some(t) => match t {
                    // セミコロンだったらreturnする
                    Token::Symbol(';') => {
                        let _ = self.output.write(
                            (t.to_xml() + "\n").as_bytes());
                        let _ = self.output.write(b"</varDec>\n");
                        return Ok(())
                    },
                    Token::Symbol(',') => {
                        let _ = self.output.write(
                            (t.to_xml() + "\n").as_bytes());
                    },
                    _ => return ErrUnexpect!(t, self.tokenizer.get_line_number())
                },
                None => return ErrReachedEnd!()
            }

            let t = MatchToken!(self.tokenizer.advance(),
                                self.tokenizer.get_line_number(),
                                Token::Keyword(Keyword::Int),
                                Token::Keyword(Keyword::Char),
                                Token::Keyword(Keyword::Boolean),
                                Token::Identifier(_));
            let _ = self.output.write((t.to_xml() + "\n").as_bytes());
        }
    }

    fn compile_statements(&mut self) -> Result<(), String> {
        // self.tokenizer.advance();
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