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

pub fn get_args() -> File {
    let matches = clap_app!(myapp =>
        (version: "0.1")
        (author: "Florian Lackner <lacknerflo@gmail.com>")
        (about: "Interprets Scheme code")
        (@arg INPUT: +required {file_is_present} "File to interpret")
    ).get_matches();

    let input_path = matches.value_of("INPUT").unwrap(); //safe because input is required
    match File::open(input_path) {
        Ok(file) => file,
        Err(err) => {
            println!("Error opening file {:?}: {:?}", input_path, err);
            exit(-1);
        }
    }
}
