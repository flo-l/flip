#![feature(conservative_impl_trait)]
#[macro_use] extern crate clap;
#[macro_use] extern crate itertools;

extern crate rustyline;
extern crate siphasher;
extern crate lalrpop_util;

#[macro_use]
mod native;

mod cli;
mod grammar;
mod value;
mod interpreter;
mod repl;
mod scope;
mod string_interner;

use std::io::Read;
use ::value::Value;

fn main() {
    if let Some(mut file) = cli::get_args() {
        let mut input = String::new();
        file.read_to_string(&mut input).expect("Couldn't read file");
        let mut interpreter = interpreter::Interpreter::new();
        let parsed = grammar::parse(&input, &mut interpreter.interner).unwrap();

        let mut result = Value::empty_list();
        for x in &parsed {
            result = interpreter.evaluate(x);
        }
        println!("=> {}", result.to_string(&interpreter.interner))
    } else {
        repl::Repl::start();
    }
}
