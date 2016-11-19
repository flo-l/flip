#[macro_use]
extern crate clap;

#[macro_use]
extern crate nom;

extern crate siphasher;

mod cli;
mod parser;
mod value;
mod interpreter;
mod repl;

fn main() {
    if let Some(file) = cli::get_args() {
        let parsed = parser::parse(file);
        let mut interpreter = interpreter::Interpreter::new(parsed);
        let ret = interpreter.start();
        println!("{}", ret);
    } else {
        repl::Repl::start();
    }
}
