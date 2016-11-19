use std::path::Path;
use std::fs::File;
use std::process::exit;

fn file_is_present(val: String) -> Result<(), String> {
    let path = Path::new(&val);
    if path.is_file() {
        Result::Ok(())
    } else {
        Result::Err("Path is no valid file".into())
    }
}

pub fn get_args() -> Option<File> {
    let matches = clap_app!(myapp =>
        (version: "0.1")
        (author: "Florian Lackner <lacknerflo@gmail.com>")
        (about: "Interprets Scheme code")
        (@arg INPUT: {file_is_present} "File to interpret")
    ).get_matches();

    matches.value_of("INPUT")
    .map(File::open)
    .map(|x| x.unwrap_or_else(|err| {
        println!("Error opening file: {}", err);
        exit(-1);
    }))
}
