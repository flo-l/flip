#[macro_use] extern crate clap;
#[macro_use] extern crate nom;

extern crate rustyline;
extern crate siphasher;

mod cli;
mod parser;
mod value;
mod interpreter;
mod repl;

use std::io::Read;

fn main() {
    if let Some(file) = cli::get_args() {
        let input: Vec<u8> = file.bytes().filter_map(Result::ok).collect();
        println!("{:?}", repl::parse_and_compile(&input));
    } else {
        repl::Repl::start();
    }
}
