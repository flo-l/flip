use std::usize;
use super::super::value::Value;
use lalrpop_util::ParseError;
use super::lexer::{Token, Error};

const EOF: usize = usize::MAX;
const INVALID_CHAR: usize = usize::MAX-1;

fn unrecognized(x: &ParseError<usize, Token, Error>) -> usize {
    if let &ParseError::UnrecognizedToken{expected: _, token: Some((pos, _, _))} = x {
        pos
    } else {
        panic!("expected: unrecognized, got: {:?}", x)
    }
}

fn invalid(x: &ParseError<usize, Token, Error>) -> usize {
    if let &ParseError::InvalidToken{location: pos} = x {
        pos
    } else {
        panic!("expected: invalid, got: {:?}", x)
    }
}

fn whatever(x: &ParseError<usize, Token, Error>) -> usize {
    match x {
        &ParseError::InvalidToken{location: pos} => pos,
        &ParseError::UnrecognizedToken{expected: _, token: Some((pos, _, _))} => pos,
        &ParseError::User{error: Error::UnexpectedToken(x)} => x,
        &ParseError::User{error: Error::UnexpectedEof} => EOF,
        &ParseError::User{error: Error::InvalidCharacter(_)} => INVALID_CHAR,
        x => panic!("got: {:?}", x),
    }
}

macro_rules! expect_error {
    // unrecognized token
    ($parser:ident, $input:expr, unrecognized: $position:expr) => (
        expect_error!($parser, $input, unrecognized, true, $position);
    );

    // invalid token
    ($parser:ident, $input:expr, invalid: $position:expr) => (
        expect_error!($parser, $input, invalid, true, $position);
    );

    // whatever
    ($parser:ident, $input:expr, $position:expr) => (
        expect_error!($parser, $input, whatever, true, $position);
    );

    // whatever wherever
    ($parser:ident, $input:expr) => (
        expect_error!($parser, $input, whatever, false, 0);
    );

    //internal
    ($parser:ident, $input:expr, $token_fn:ident, $use_pos:expr, $position:expr) => (
        let input = &*$input;
        let result = $parser(input);

        if let Err(ref err) = result {
            if $use_pos && $token_fn(err) != $position {
                panic!("for input: {:?}, got error {:?} at pos {}, expected pos: {}", input, result, $token_fn(err), $position)
            }
        } else {
            panic!("expected error for string: {:?}, got: {:?}", input, result);
        }
    );
}

macro_rules! expect_ok {
    // with rest
    ($parser:ident, $input:expr, $expected:expr) => (
        let input = &*$input;
        let expected = $expected;
        let result = $parser(input);
        if let Ok(v) = result {
            if v != expected {
                panic!("parser ok, but input: {:?} got: Ok({:?}), expected: {:?}", input, v, &expected);
            }
        } else {
            panic!("input: {:?} got: {:?}, expected: {:?}", input, result, &expected);
        }
    );
}

#[test]
fn bool() {
    use super::parse_Symbol;
    expect_ok!(parse_Symbol, "true", Value::new_bool(true));
    expect_ok!(parse_Symbol, "false", Value::new_bool(false));
    expect_ok!(parse_Symbol, "trude", Value::new_symbol("trude"));
    expect_ok!(parse_Symbol, "fale", Value::new_symbol("fale"));
}

#[test]
fn char() {
    use super::parse_Char;
    use std::char;
    use std::iter;
    let mut input: String = "#\\".into();
    let printlable_asci = ('!' as u32)..('~' as u32) + 1;
    for x in printlable_asci {
        let c = char::from_u32(x).expect(&format!("tried to create invalid char with: 0x{:X}", x));
        input.push(c);
        { expect_ok!(parse_Char, input, Value::new_char(c)); }
        input.pop();
    }

    let invalid_asci = (0..('!' as u32)).chain(iter::once(127));
    for x in invalid_asci {
        let c = char::from_u32(x).expect(&format!("tried to create invalid char with: 0x{:X}", x));
        input.push(c);
        { expect_error!(parse_Char, input); }
        input.pop();
    }

    // special forms
    expect_ok!(parse_Char, r"#\\s", Value::new_char(' '));
    expect_ok!(parse_Char, r"#\\t", Value::new_char('\t'));
    expect_ok!(parse_Char, r"#\\n", Value::new_char('\n'));
    expect_ok!(parse_Char, r"#\\", Value::new_char('\\'));

    expect_error!(parse_Char, r"#\ ", 0);
    expect_error!(parse_Char, "#\\\n", 0);
}

#[test]
fn integer() {
    use super::parse_Integer;
    expect_ok!(parse_Integer, "007", Value::new_integer(7));
    expect_ok!(parse_Integer, "-007", Value::new_integer(-7));
    expect_ok!(parse_Integer, "123456789", Value::new_integer(123456789));
    expect_ok!(parse_Integer, "-123456789", Value::new_integer(-123456789));

    expect_error!(parse_Integer, "123b456789", 3);
    expect_error!(parse_Integer, "123456789c", 9);
    expect_error!(parse_Integer, "00-7", 2);
    expect_error!(parse_Integer, "a123456789", 0);
    expect_error!(parse_Integer, "--7", 0);
}

#[test]
fn symbol() {
    use super::parse_Symbol;
    expect_ok!(parse_Symbol, "+", Value::new_symbol("+"));
    expect_ok!(parse_Symbol, "-", Value::new_symbol("-"));
    expect_ok!(parse_Symbol, "#", Value::new_symbol("#"));
    expect_ok!(parse_Symbol, "a1a", Value::new_symbol("a1a"));
    expect_ok!(parse_Symbol, "num->str", Value::new_symbol("num->str"));
    expect_ok!(parse_Symbol, "//", Value::new_symbol("//"));

    // error is at 1 bc lexer tries to lex integer
    expect_error!(parse_Symbol, "1a", 1);
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

#[test]
fn string() {
    use super::parse_String;
    use std::iter;
    fn q(s: &str) -> String {
        iter::once('"')
        .chain(s.chars())
        .chain(iter::once('"'))
        .collect()
    }

    macro_rules! expect_str_ok {
        ($s:expr, $e:expr) => ({
            let s = q($s);
            println!("string: {}", s);
            expect_ok!(parse_String, s, Value::new_string($e));
        });

        ($s:expr) => (expect_str_ok!($s, $s));
    }

    expect_str_ok!("");
    expect_str_ok!("abc");
    expect_str_ok!("Hello, World!!");
    expect_str_ok!("\n");
    expect_str_ok!(r"\n", "\n");
    expect_str_ok!("\t");
    expect_str_ok!(r"\t", "\t");
    expect_str_ok!(r"\\\\", "\\\\");
    expect_str_ok!(r#"Hi there: \" \\ \n \t"#, "Hi there: \" \\ \n \t");

    expect_error!(parse_String, "\"", EOF);
    expect_error!(parse_String, "\"❤\"", INVALID_CHAR); // non ascii
}

#[test]
fn pair() {
    use super::parse_Pair;
    let t = Value::new_bool(true);
    let f = Value::new_bool(false);
    let e = Value::empty_list();

    expect_ok!(parse_Pair, "(true . false)", Value::new_pair(t.clone(), f.clone()));
    expect_ok!(parse_Pair, "(true . (false . ()))", Value::new_pair(t, Value::new_pair(f.clone(), e.clone())));

    expect_error!(parse_Pair, "(1 .)", 4);
    expect_error!(parse_Pair, "(. 2)", 1);
    expect_error!(parse_Pair, "(1 . 2 3)", 7);
    expect_error!(parse_Pair, "(1 . 2 . 3)", 7);
    expect_error!(parse_Pair, "(1 2 . 3)", 3);
}
