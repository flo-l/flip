#[cfg(test)]
mod tests;

mod parser;
mod lexer;
mod error;
pub mod error_printing;

use std::mem;
use lalrpop_util::ParseError;
use ::value::Value;
use ::string_interner::StringInterner;
use tail_calls::check_tail_calls;

pub fn parse<'input>(input: &'input str, interner: &mut StringInterner)
-> Result<Value, ::lalrpop_util::ParseError<usize, lexer::Token<'input>, error::Error>> {
    let tokenizer = lexer::Tokenizer::new(input);
    let parsed = parser::parse_TopLevelItem(input, true, interner, tokenizer);

    match parsed {
        Ok(value) => {
            let code = &[value];
            if !check_tail_calls(code) {
                Err(ParseError::User{error: error::Error::RecurInNonTailPosition})
            } else {
                Ok(code[0].clone())

            }
        }
        err => err,
    }
}

pub fn parse_integer<'input>(input: &'input str)
-> Result<Value, ::lalrpop_util::ParseError<usize, lexer::Token<'input>, error::Error>> {
    // safe because we give parse_Integer false, so it knows the pointer is invalid
    let fake_interner: &mut StringInterner = unsafe { mem::transmute(0usize) };
    let tokenizer = lexer::Tokenizer::new(input);
    parser::parse_Integer(input, false, fake_interner, tokenizer)
}
