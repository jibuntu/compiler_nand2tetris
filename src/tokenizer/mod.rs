//! Tokenizerは入力からTokenを取り出す。
//! TokenはToken構造体。
//! Tokenizerはiteratorをimplementしているため、for文で使える

use std::io::Read;
use std::io::BufReader;
use std::iter::Iterator;

mod token;
use token::Token;


pub struct Tokenizer<R> {
    stream: BufReader<R>
}

impl<R: Read> Tokenizer<R> {
    pub fn new(stream: R) -> Tokenizer<R> {
        Tokenizer {
            stream: BufReader::new(stream)
        }
    }
}
