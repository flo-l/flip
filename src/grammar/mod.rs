#[cfg(test)]
mod tests;

mod parser;
mod lexer;
mod error;
pub mod error_printing;

use std::mem;
use ::value::Value;
use ::string_interner::StringInterner;

pub use self::lexer::escape_char;

static NO_INTERNER_ERROR_STRING: &'static str = "internal error: interner not set";

pub fn parse<'input>(input: &'input str, interner: &mut StringInterner)
-> Result<Vec<Value>, ::lalrpop_util::ParseError<usize, lexer::Token<'input>, error::Error>> {
    let tokenizer = lexer::Tokenizer::new(input);
    parser::parse_TopLevelItem(input, true, interner, tokenizer)
}

pub fn parse_integer<'input>(input: &'input str)
-> Result<Value, ::lalrpop_util::ParseError<usize, lexer::Token<'input>, error::Error>> {
    // safe because we give parse_Integer false, so it knows the pointer is invalid
    let fake_interner: &mut StringInterner = unsafe { mem::transmute(0usize) };
    let tokenizer = lexer::Tokenizer::new(input);
    parser::parse_Integer(input, false, fake_interner, tokenizer)
}
