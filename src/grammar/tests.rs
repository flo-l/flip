use super::parser::{parse_Bool};
use nom::{IResult, Err, ErrorKind};
use super::super::value::Value;
use lalrpop_util::ParseError;

macro_rules! expect_error {
    // without error kind
    ($parser:ident, $input:expr, $position:expr) => (
        let input = &*$input;
        let error = ParseError::InvalidToken {
            location: $position,
        };
        assert_eq!(
            $parser(input),
            Result::Err(error)
        );
    );
}

macro_rules! expect_ok {
    // with rest
    ($parser:ident, $input:expr, $expected:expr) => (
        let input = &*$input;
        assert_eq!(
            $parser(input),
            Result::Ok($expected)
        );
    );
}

#[test]
fn bool() {
    expect_ok!(parse_Bool, "true", Value::new_bool(true));
    expect_ok!(parse_Bool, "false", Value::new_bool(false));
    expect_error!(parse_Bool, "trude", 0);
    expect_error!(parse_Bool, "fale", 0);
}
