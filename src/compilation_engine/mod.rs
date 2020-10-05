use std::io::{Read, Write, Seek};

use super::tokenizer::Tokenizer;
use super::tokenizer::token::Token;
use super::tokenizer::token::Keyword;


macro_rules! ErrReachedEnd {
    () => {
        Err("reached end of file while parsing".to_string())
    };
}

#[cfg(debug_assertions)]
macro_rules! ErrUnexpect {
    ($token:expr, $line_number:expr) => {
        Err(format!("unexpected token: '{}' at line {}. DEBUG: line {}", 
                    $token.to_string(),
                    $line_number,
                    line!()))
    };
}

#[cfg(not(debug_assertions))]
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

        loop {
            match self.tokenizer.get_current_token() {
                Some(t) => match t {
                    Token::Symbol('}') => {
                        let _ = self.output.write((t.to_xml() + "\n").as_bytes());
                        break
                    },
                    _ => self.compile_statements()?
                    //_ => return ErrUnexpect!(t, self.tokenizer.get_line_number())
                },
                None => return ErrReachedEnd!()
            }

// self.compile_statementsですでにadvanceしているので、ここでadvanceをすると
// 二重になってしまう(ので、エラーになる)
//            self.tokenizer.advance();
        }

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
        let _ = self.output.write(b"<statements>\n");

        loop {
            match self.tokenizer.get_current_token() {
                Some(t) => match t {
                    // TODO: do, while, return, ifを実装する
                    Token::Keyword(Keyword::Do) => self.compile_do()?,
                    Token::Keyword(Keyword::Let) => self.compile_let_statement()?,
                    // Token::Keyword(Keyword::While) => self.compile_let_statement()?,
                    // Token::Keyword(Keyword::Return) => self.compile_let_statement()?,
                    // Token::Keyword(Keyword::If) => self.compile_let_statement()?,
                    _ => break
                },
                None => return ErrReachedEnd!()
            }

            self.tokenizer.advance();
        }

        let _ = self.output.write(b"</statements>\n");
        Ok(())
    }

    /*
     * 'do' subroutineCall ';'
     * */
    fn compile_do(&mut self) -> Result<(), String> {
        let _ = self.output.write(b"<doStatement>\n");

        let t = MatchToken!(self.tokenizer.get_current_token(),
                            self.tokenizer.get_line_number(),
                            Token::Keyword(Keyword::Do));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

        let t = MatchToken!(self.tokenizer.advance(),
                            self.tokenizer.get_line_number(),
                            Token::Identifier(_));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

        match self.tokenizer.advance() {
            Some(t) => match t {
                // 関数呼び出しのとき
                Token::Symbol('(') => {
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());
                    self.compile_expression_list()?;

                    let t = MatchToken!(self.tokenizer.get_current_token(),
                                        self.tokenizer.get_line_number(),
                                        Token::Symbol(')'));
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());

                    self.tokenizer.advance();
                },
                // メソッド呼び出しのとき
                Token::Symbol('.') => {
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());

                    let t = MatchToken!(self.tokenizer.advance(),
                                        self.tokenizer.get_line_number(),
                                        Token::Identifier(_));
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());

                    let t = MatchToken!(self.tokenizer.advance(),
                                        self.tokenizer.get_line_number(),
                                        Token::Symbol('('));
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());

                    self.compile_expression_list()?;

                    let t = MatchToken!(self.tokenizer.get_current_token(),
                                        self.tokenizer.get_line_number(),
                                        Token::Symbol(')'));
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());

                    self.tokenizer.advance();
                },
                _ => return ErrUnexpect!(t, self.tokenizer.get_line_number())
            },
            None => return ErrReachedEnd!()
        }

        let t = MatchToken!(self.tokenizer.get_current_token(),
                            self.tokenizer.get_line_number(),
                            Token::Symbol(';'));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

        let _ = self.output.write(b"</doStatement>\n");

        Ok(())
    }

    /*
     * 'let' varName ('[' expression ']')? '=' expression ';'
     */
    fn compile_let_statement(&mut self) -> Result<(), String> {
        let _ = self.output.write(b"<letStatement>\n");
 
        let t = MatchToken!(self.tokenizer.get_current_token(),
                            self.tokenizer.get_line_number(),
                            Token::Keyword(Keyword::Let));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());
 
        let t = MatchToken!(self.tokenizer.advance(),
                            self.tokenizer.get_line_number(),
                            Token::Identifier(_));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());
 
        let t = MatchToken!(self.tokenizer.advance(),
                            self.tokenizer.get_line_number(),
                            Token::Symbol('='));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());

        self.tokenizer.advance();
        self.compile_expression()?;

        let t = MatchToken!(self.tokenizer.get_current_token(),
                            self.tokenizer.get_line_number(),
                            Token::Symbol(';'));
        let _ = self.output.write((t.to_xml() + "\n").as_bytes());
 
        let _ = self.output.write(b"</letStatement>\n");
        Ok(())
    }

    /*
     * term (op term)* 
     * */
    fn compile_expression(&mut self) -> Result<(), String> {
        let _ = self.output.write(b"<expression>\n");

        self.compile_term()?;
        
        loop {
            match self.tokenizer.get_current_token() {
                Some(t) => match t {
                    Token::Symbol('+') | Token::Symbol('-') |
                    Token::Symbol('*') | Token::Symbol('/') |
                    Token::Symbol('&') | Token::Symbol('|') |
                    Token::Symbol('<') | Token::Symbol('>') |
                    Token::Symbol('=') => {
                        let _ = self.output.write((t.to_xml() + "\n").as_bytes());
                        self.tokenizer.advance();
                        self.compile_term()?;
                    },
                    _ => break
                },
                None => return ErrReachedEnd!()
            }
        }

        let _ = self.output.write(b"</expression>\n");
        Ok(())
    }

    /*
     * interConstant | stringConstant | keywordConstant |
     * varName '[' expression ']' | subroutineCall | '(' expression ')' |
     * unaryOp term 
     * */
    fn compile_term(&mut self) -> Result<(), String> {
        let _ = self.output.write(b"<term>\n");

        match self.tokenizer.get_current_token() {
            Some(t) => match t {
                // これらのトークンは先読みが不要なので早期リターンする
                Token::Keyword(Keyword::True) |
                Token::Keyword(Keyword::False) |
                Token::Keyword(Keyword::Null) |
                Token::Keyword(Keyword::This) |
                Token::Integer(_) |
                Token::String(_) => {
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());
                    self.tokenizer.advance();

                    let _ = self.output.write(b"</term>\n");
                    return Ok(())
                },
                Token::Identifier(_) => {
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());
                },
                // カッコで囲われた式のとき
                Token::Symbol('(') => {
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());

                    self.tokenizer.advance();
                    self.compile_expression()?;
                    
                    let t = MatchToken!(self.tokenizer.get_current_token(),
                                        self.tokenizer.get_line_number(),
                                        Token::Symbol(')'));
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());
                },
                // 単項演算子のとき
                Token::Symbol('-') | Token::Symbol('~') => {
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());

                    self.tokenizer.advance();
                    self.compile_term()?;

                    let _ = self.output.write(b"</term>\n");
                    return Ok(())
                },
                _ => return ErrUnexpect!(t, self.tokenizer.get_line_number())
            },
            None => return ErrReachedEnd!()
        }

        match self.tokenizer.advance() {
            Some(t) => match t {
                // 配列のとき
                Token::Symbol('[') => {
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());

                    self.tokenizer.advance();
                    self.compile_expression()?;

                    let t = MatchToken!(self.tokenizer.get_current_token(),
                                        self.tokenizer.get_line_number(),
                                        Token::Symbol(']'));
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());

                    self.tokenizer.advance();
                },
                // 関数呼び出しのとき
                Token::Symbol('(') => {
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());
                    self.compile_expression_list()?;

                    let t = MatchToken!(self.tokenizer.get_current_token(),
                                        self.tokenizer.get_line_number(),
                                        Token::Symbol(')'));
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());

                    self.tokenizer.advance();
                },
                // メソッド呼び出しのとき
                Token::Symbol('.') => {
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());

                    let t = MatchToken!(self.tokenizer.advance(),
                                        self.tokenizer.get_line_number(),
                                        Token::Identifier(_));
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());

                    let t = MatchToken!(self.tokenizer.advance(),
                                        self.tokenizer.get_line_number(),
                                        Token::Symbol('('));
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());

                    self.compile_expression_list()?;

                    let t = MatchToken!(self.tokenizer.get_current_token(),
                                        self.tokenizer.get_line_number(),
                                        Token::Symbol(')'));
                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());

                    self.tokenizer.advance();
                },
                // それ以外はただの変数として扱う
                _ => {
                    // すでにxmlに出力しているので特にすることはない
                }
            },
            None => return ErrReachedEnd!()
        }

        let _ = self.output.write(b"</term>\n");
        Ok(())
    }

    /*
     * (expression (',' expression)* )?
     * */
    fn compile_expression_list(&mut self) -> Result<(), String> {
        let _ = self.output.write(b"<expressionList>\n");

        loop {
            match self.tokenizer.advance() {
                Some(t) => match t {
                    Token::Symbol(')') => {
                        break
                    },
                    _ => {
                        self.compile_expression()?;
                        match self.tokenizer.get_current_token() {
                            Some(t) => match t {
                                Token::Symbol(')') => {
                                    break
                                },
                                Token::Symbol(',') => {
                                    let _ = self.output.write((t.to_xml() + "\n").as_bytes());
                                },
                                _ => return ErrUnexpect!(t, self.tokenizer.get_line_number())
                            },
                            None => return ErrReachedEnd!()
                        }
                    }
                },
                None => return ErrReachedEnd!()
            }
        }

        let _ = self.output.write(b"</expressionList>\n");
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
