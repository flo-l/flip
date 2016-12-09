#[macro_use] extern crate clap;
#[macro_use] extern crate nom;

//TODO remove this (also cargo.toml)
extern crate regex;

extern crate rustyline;
extern crate siphasher;
extern crate lalrpop_util;

mod cli;
mod grammar;
mod value;
mod interpreter;
mod repl;
mod scope;
mod native;

use std::io::Read;

fn main() {
    if let Some(mut file) = cli::get_args() {
        let mut input = String::new();
        file.read_to_string(&mut input).expect("Couldn't read file");
        let parsed = grammar::parse(&input).unwrap();
        let mut interpreter = interpreter::Interpreter::new();
        println!("{:?}", interpreter.evaluate(&parsed));
    } else {
        repl::Repl::start();
    }
}
