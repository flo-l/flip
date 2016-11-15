#[macro_use]
extern crate clap;

#[macro_use]
extern crate nom;

mod cli;
mod parser;
mod ir;
mod interpreter;

fn main() {
    let input_file = cli::get_args();
    let parsed = parser::parse(input_file);
    println!("Parsed: {}", parsed);

    let mut interpreter = interpreter::Interpreter::new(parsed);
    let ret = interpreter.start();
    println!("{}", ret);
}
