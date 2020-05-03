use std::fs::File;
use std::io::BufReader;
use std::env;

mod tokenizer;
use tokenizer::Tokenizer;
mod compilation_engine;
use compilation_engine::CompilationEngine;


fn main() {
    let f = match env::args().nth(1) {
        Some(f) => match File::open(f) {
            Ok(f) => f,
            Err(_) => return println!("ファイルが開けません")
        }
        None => return println!("ファイル名を指定してください")
    };

    let o = match env::args().nth(2) {
        Some(f) => match File::create(f) {
            Ok(f) => f,
            Err(_) => return println!("ファイルが開けません")
        }
        None => return println!("出力するファイル名を指定してください")
    };

    let reader = BufReader::new(f);
    let t = Tokenizer::new(reader);
    let mut c = CompilationEngine::new(t, o);

    c.tokenizer.advance();
    match c.compile_class() {
        Ok(()) => (),
        Err(e) => println!("error: {}", e)
    }
}
