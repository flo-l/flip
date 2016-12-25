use rustyline;
use std::iter;
use std::collections::btree_set::BTreeSet;
use ::interpreter;
use ::grammar::{self, error_printing};

pub struct Repl {}

impl Repl {
    pub fn start() {
        let quit = "(quit)";
        let break_chars: BTreeSet<char> = vec![' ', '(', '\''].into_iter().collect();
        let mut rl = rustyline::Editor::<IdentCompleter>::new();
        rl.add_history_entry(quit);

        let mut interpreter = interpreter::Interpreter::new();

        loop {
            let idents: Vec<String> = interpreter.current_scope.symbol_ids()
            .into_iter()
            .filter_map(|id| interpreter.interner.lookup(id).map(Into::into))
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

            // strip \n at the end
            let parsed = grammar::parse(&line, &mut interpreter.interner);
            match parsed {
                Ok(value) => println!("=> {}", interpreter.evaluate(&value).to_string(&interpreter.interner)),
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

        let symbol_matches =
        self.ident_list.iter()
        .filter(|&ident| ident.starts_with(word))
        .cloned();

        // if word is just whitespaces return closing parens as first result
        let matches: Vec<String> = if word.chars().all(|c| c == ' ') {
            iter::once(close_params(line))
            .chain(symbol_matches)
            .collect()
        // else return closing parens as last result
        } else {
            symbol_matches
            .chain(iter::once(close_params(line)))
            .collect()
        };

        Ok((start, matches))
    }
}

fn close_params(line: &str) -> String {
    let unclosed_parens = line.chars()
    .fold(0, |n, c| if c == '(' { n + 1 } else if c == ')' { n - 1 } else { n });
    let missing_parens: String = iter::repeat(')').take(unclosed_parens).collect();
    missing_parens
}
