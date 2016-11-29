use nom::{IResult, Err, ErrorKind};
use super::super::value::Value;
use lalrpop_util::ParseError;

fn unrecognized(x: &ParseError<usize, (usize, &str), ()>) -> usize {
    if let &ParseError::UnrecognizedToken{expected: _, token: Some((pos, _, _))} = x {
        pos
    } else {
        panic!("expected: unrecognized, got: {:?}", x)
    }
}

fn invalid(x: &ParseError<usize, (usize, &str), ()>) -> usize {
    if let &ParseError::InvalidToken{location: pos} = x {
        pos
    } else {
        panic!("expected: invalid, got: {:?}", x)
    }
}

macro_rules! expect_error {
    // unrecognized token
    ($parser:ident, $input:expr, unrecognized: $position:expr) => (
        expect_error!($parser, $input, unrecognized, $position);
    );

    // invalid token
    ($parser:ident, $input:expr, invalid: $position:expr) => (
        expect_error!($parser, $input, invalid, $position);
    );

    //internal
    ($parser:ident, $input:expr, $token_fn:ident, $position:expr) => (
        let input = &*$input;
        let result = $parser(input);

        if let Err(ref err) = result {
            if $token_fn(err) != $position {
                panic!("ecpected: {}, got: {}", $position, $token_fn(err))
            }
        } else {
            panic!("string: {:?}, {:?}", input, result);
        }
    );
}

macro_rules! expect_ok {
    // with rest
    ($parser:ident, $input:expr, $expected:expr) => (
        let input = &*$input;
        let result = $parser(input);

        if let Ok(v) = result {
            assert_eq!(v, $expected)
        } else {
            panic!("input: {:?} got: {:?}, expected: {:?}", input, result, $expected);
        }
    );
}

#[test]
fn bool() {
    use super::parser::parse_Bool;
    expect_ok!(parse_Bool, "true", Value::new_bool(true));
    expect_ok!(parse_Bool, "false", Value::new_bool(false));
    expect_error!(parse_Bool, "trude", invalid: 0);
    expect_error!(parse_Bool, "fale", invalid: 0);
}

#[test]
fn char() {
    use super::parser::parse_Char;
    use regex::Regex;
    let whitespace = Regex::new(r"\S").unwrap();

    use std::u8;
    for x in 0..127 {
        let input = String::from_utf8(vec!['#' as u8, '\\' as u8, x]).unwrap();
        if whitespace.is_match(&input) { continue }
        expect_ok!(parse_Char, input, Value::new_char(x as char));
    }
    expect_error!(parse_Char, "#\\", invalid: 0);

    //TODO test non ASCI chars
}

#[test]
fn integer() {
    use super::parser::parse_Integer;
    expect_ok!(parse_Integer, "007", Value::new_integer(7));
    expect_ok!(parse_Integer, "-007", Value::new_integer(-7));
    expect_ok!(parse_Integer, "123456789", Value::new_integer(123456789));
    expect_ok!(parse_Integer, "-123456789", Value::new_integer(-123456789));

    expect_error!(parse_Integer, "123b456789", invalid: 3);
    expect_error!(parse_Integer, "123456789c", invalid: 9);
    expect_error!(parse_Integer, "00-7", unrecognized: 2);
    expect_error!(parse_Integer, "a123456789", invalid: 0);
    expect_error!(parse_Integer, "--7", unrecognized: 1);
}

// TODO
/*
// test item error
expect_ok!(integer, "123b456789", Value::new_integer(123), "b456789");
expect_ok!(integer, "123456789c", Value::new_integer(123456789), "c");
expect_ok!(integer, "00-7", Value::new_integer(0), "-7");
expect_error!(integer, "a123456789", 0);
expect_error!(integer, "--7", 1);
*/
