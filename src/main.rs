#[macro_use]
extern crate clap;

#[macro_use]
extern crate nom;

extern crate siphasher;

mod cli;
mod parser;
mod value;
mod interpreter;

fn main() {
    let input_file = cli::get_args();
    let parsed = parser::parse(input_file);
    println!("Parsed: {}", parsed);

    let mut interpreter = interpreter::Interpreter::new(parsed);
    let ret = interpreter.start();
    println!("{}", ret);
}
