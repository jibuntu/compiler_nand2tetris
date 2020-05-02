use std::fs::File;
use std::io::BufReader;
use std::env;

mod tokenizer;
use tokenizer::Tokenizer;


fn main() {
    let f = match env::args().nth(1) {
        Some(f) => match File::open(f) {
            Ok(f) => f,
            Err(_) => return println!("ファイルが開けません")
        }
        None => return println!("ファイル名を指定してください")
    };

    let reader = BufReader::new(f);
    println!("<tokens>");
    for token in Tokenizer::new(reader) {
        println!("  {}", token.to_xml());
    }
    println!("</tokens>");
}
