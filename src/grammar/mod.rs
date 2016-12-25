#[cfg(test)]
mod tests;

mod parser;
mod lexer;
pub mod error_printing;

use std::mem;
use ::value::Value;
use ::string_interner::StringInterner;

pub fn parse<'input>(input: &'input str, interner: &mut StringInterner)
-> Result<Value, ::lalrpop_util::ParseError<usize, lexer::Token<'input>, lexer::Error>> {
    let tokenizer = lexer::Tokenizer::new(input);
    let parsed = parser::parse_TopLevelItem(input, true, interner, tokenizer);
    parsed
}

pub fn parse_integer<'input>(input: &'input str)
-> Result<Value, ::lalrpop_util::ParseError<usize, lexer::Token<'input>, lexer::Error>> {
    // safe because we give parse_Integer false, so it knows the pointer is invalid
    let fake_interner: &mut StringInterner = unsafe { mem::transmute(0usize) };
    let tokenizer = lexer::Tokenizer::new(input);
    parser::parse_Integer(input, false, fake_interner, tokenizer)
}
