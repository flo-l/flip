#[macro_use]
extern crate clap;

#[macro_use]
extern crate nom;

mod cli;
mod parser;

fn main() {
    let input_file = cli::get_args();
    let parsed = parser::parse(input_file);

    println!("{:?}", parsed);
}
