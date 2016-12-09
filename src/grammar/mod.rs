#[cfg(test)]
mod tests;

mod parser;
mod lexer;

use ::value::Value;

#[allow(dead_code)]
fn unescape_string(input: &str) -> String {
    let mut chars = input.chars();
    let mut s = String::with_capacity(input.len());

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                match next {
                    'n' => s.push('\n'),
                    't' => s.push('\t'),
                    '"' => s.push('"'),
                    's' => s.push(' '),
                    '\\' => s.push('\\'),
                    x => {
                        s.push('\\');
                        s.push(x);
                    }
                }
                continue;
            }
        }
        s.push(c);
    }
    s
}

pub fn parse<'input>(input: &'input str)
-> Result<Value, ::lalrpop_util::ParseError<usize, lexer::Token<'input>, lexer::Error>> {
    let tokenizer = lexer::Tokenizer::new(input);
    parser::parse_TopLevelItem(input, tokenizer)
}
