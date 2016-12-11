use rustyline;
use std::iter;
use std::collections::btree_set::BTreeSet;
use super::interpreter;
use super::grammar::{self, error_printing};
use super::scope::SymbolIterator;

pub struct Repl {}

impl Repl {
    pub fn start() {
        let quit = "(quit)";
        let break_chars: BTreeSet<char> = vec![' ', '('].into_iter().collect();
        let mut rl = rustyline::Editor::<IdentCompleter>::new();
        rl.add_history_entry(quit);

        let mut interpreter = interpreter::Interpreter::new();

        loop {
            let idents = SymbolIterator::new(&interpreter.current_scope)
            .map(|(_, &(ref s, _))| s.clone())
            .collect();

            let completer = IdentCompleter {
                break_chars: &break_chars,
                ident_list: idents
            };

            rl.set_completer(Some(completer));

            let line = rl.readline(">> ");
            let line = if line.is_err() { return } else { line.unwrap() };
            if line == quit { return }
            rl.add_history_entry(&line);

            let parsed = grammar::parse(&line);
            match parsed {
                Ok(value) => println!("=> {}", interpreter.evaluate(&value)),
                Err(ref err)  => println!("{}", error_printing::create_error_message(&line, err)),
            }
        }
    }
}

struct IdentCompleter<'a> {
    break_chars: &'a BTreeSet<char>,
    ident_list: Vec<String>,
}

impl<'a> rustyline::completion::Completer for IdentCompleter<'a> {
    fn complete(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<String>)> {
        let (start, word) = rustyline::completion::extract_word(line, pos, &self.break_chars);
        let matches: Vec<String> = self.ident_list.iter()
        .filter(|&ident| ident.starts_with(word))
        .cloned()
        //.chain(iter::once(close_params(line)))
        .collect();
        Ok((start, matches))
    }
}

fn close_params(line: &str) -> String {
    let unclosed_parens = line.chars()
    .fold(0, |n, c| if c == '(' { n + 1 } else if c == ')' { n - 1 } else { n });
    let missing_parens: String = iter::repeat(')').take(unclosed_parens).collect();
    missing_parens
}
