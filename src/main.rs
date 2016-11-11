#[macro_use]
extern crate clap;

mod cli;


fn main() {
    let input_file = cli::get_args();

    println!("{:?}", input_file);
}
