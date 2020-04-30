use std::env;

mod tokenizer;
use tokenizer::Tokenizer;


fn main() {
    let filename = match env::args().nth(1) {
        Some(f) => f,
        None => return println!("ファイル名を指定してください")
    };
}
