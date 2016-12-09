#[cfg(test)]
mod tests;

mod parser;
mod lexer;

use ::value::Value;

pub fn parse<'input>(input: &'input str)
-> Result<Value, ::lalrpop_util::ParseError<usize, lexer::Token<'input>, lexer::Error>> {
    let tokenizer = lexer::Tokenizer::new(input);
    parser::parse_TopLevelItem(input, tokenizer)
}
