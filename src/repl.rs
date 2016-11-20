use super::value;
use super::interpreter;
use super::parser;
use rustyline::{self, line_buffer};
use std::iter;

pub struct Repl {}

impl Repl {
    pub fn start() {
        let quit = "(quit)";
        let mut rl = rustyline::Editor::<ParensCloser>::new();
        rl.add_history_entry(quit);
        rl.set_completer(Some(ParensCloser{}));

        loop {
            let line = rl.readline(">> ");
            let line = if line.is_err() { return } else { line.unwrap() };
            if line == quit { return }
            let result = parse_and_compile(line.as_bytes());
            rl.add_history_entry(&line);
            match result {
                Ok(value) => println!("=> {}", value),
                Err(err)  => println!("{}", err),
            }
        }
    }
}

struct ParensCloser {}

impl rustyline::completion::Completer for ParensCloser {
    fn complete(&self, line: &str, _: usize) -> rustyline::Result<(usize, Vec<String>)> {
        let unclosed_parens = line.chars()
        .fold(0, |n, c| if c == '(' { n + 1 } else if c == ')' { n - 1 } else { n });
        let missing_parens: String = iter::repeat(')').take(unclosed_parens).collect();
        Result::Ok((line.len(), vec![missing_parens]))
    }

    fn update(&self, line: &mut line_buffer::LineBuffer, _: usize, elected: &str) {
        let len = line.len();
        line.replace(len, len, elected);
    }}

pub fn parse_and_compile(input: &[u8]) -> Result<value::Value, String>
{
    let parsed = parser::parse(input)?;
    let mut interpreter = interpreter::Interpreter::new(parsed);
    Ok(interpreter.start())
}
