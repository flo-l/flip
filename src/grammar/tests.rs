use std::usize;
use lalrpop_util::ParseError;
use ::value::Value;
use super::parse;
use super::lexer::{Token, Error};

const EOF: usize = usize::MAX;

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
        &ParseError::User{error: Error::InvalidCharacter(x)} => x,
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
            panic!("expected error for string: '{}', got: {:?}", input, result);
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
    expect_ok!(parse, "true", Value::new_bool(true));
    expect_ok!(parse, "false", Value::new_bool(false));
    expect_ok!(parse, "trude", Value::new_symbol("trude"));
    expect_ok!(parse, "fale", Value::new_symbol("fale"));
}

#[test]
fn char() {
    use std::char;
    use std::iter;
    let mut input: String = "#\\".into();
    let printlable_asci = ('!' as u32)..('~' as u32) + 1;
    for x in printlable_asci {
        let c = char::from_u32(x).expect(&format!("tried to create invalid char with: 0x{:X}", x));
        input.push(c);
        { expect_ok!(parse, input, Value::new_char(c)); }
        input.pop();
    }

    let invalid_asci = (0..('!' as u32)).chain(iter::once(127));
    for x in invalid_asci {
        let c = char::from_u32(x).expect(&format!("tried to create invalid char with: 0x{:X}", x));
        input.push(c);
        if c != '\n' && c != '\t' && c != ' ' { expect_error!(parse, input); }
        input.pop();
    }

    // special forms
    expect_ok!(parse, r"#\\s", Value::new_char(' '));
    expect_ok!(parse, r"#\\t", Value::new_char('\t'));
    expect_ok!(parse, r"#\\n", Value::new_char('\n'));
    expect_ok!(parse, r"#\\", Value::new_char('\\'));
    expect_ok!(parse, r"#\", Value::new_symbol(r"#\"));

    expect_error!(parse, "#\\\0", 2);
}

#[test]
fn integer() {
    expect_ok!(parse, "007", Value::new_integer(7));
    expect_ok!(parse, "-007", Value::new_integer(-7));
    expect_ok!(parse, "123456789", Value::new_integer(123456789));
    expect_ok!(parse, "-123456789", Value::new_integer(-123456789));

    expect_error!(parse, "123b456789", 3);
    expect_error!(parse, "123456789c", 9);
    expect_error!(parse, "00-7", 2);
    expect_ok!(parse, "a123456789", Value::new_symbol("a123456789"));
    expect_ok!(parse, "--7", Value::new_symbol("--7"));
}

#[test]
fn symbol() {
    expect_ok!(parse, "+", Value::new_symbol("+"));
    expect_ok!(parse, "-", Value::new_symbol("-"));
    expect_ok!(parse, "#", Value::new_symbol("#"));
    expect_ok!(parse, "a1a", Value::new_symbol("a1a"));
    expect_ok!(parse, "num->str", Value::new_symbol("num->str"));
    expect_ok!(parse, "//", Value::new_symbol("//"));

    // error is at 1 bc lexer tries to lex integer
    expect_error!(parse, "1a", 1);
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
            expect_ok!(parse, s, Value::new_string($e));
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

    expect_error!(parse, "\"", EOF);
    expect_error!(parse, "\"❤\"", 1); // non ascii
}

#[test]
fn pair() {
    let t = Value::new_bool(true);
    let f = Value::new_bool(false);
    let e = Value::empty_list();

    expect_ok!(parse, "(true . false)", Value::new_pair(t.clone(), f.clone()));
    expect_ok!(parse, "(true . (false . ()))", Value::new_pair(t, Value::new_pair(f.clone(), e.clone())));

    expect_error!(parse, "(1 .)", 4);
    expect_error!(parse, "(. 2)", 1);
    expect_error!(parse, "(1 . 2 3)", 7);
    expect_error!(parse, "(1 . 2 . 3)", 7);
    expect_error!(parse, "(1 2 . 3)", 5);
}

#[test]
fn list() {
    expect_ok!(parse, "()", Value::empty_list());
    expect_ok!(parse, r#"(1 "2" (3 . 4))"#, Value::new_list(&vec![Value::new_integer(1), Value::new_string("2"), Value::new_pair(Value::new_integer(3), Value::new_integer(4))]));
    expect_ok!(parse, "(() ())", Value::new_list(&vec![Value::empty_list(), Value::empty_list()]));

    expect_error!(parse, "(( ())");
}

#[test]
fn quote() {
    fn quoted(v: Value) -> Value { Value::new_list(&[Value::new_symbol("quote"), v]) }
    expect_ok!(parse, "'()", quoted(Value::empty_list()));
    expect_ok!(parse, "'1", quoted(Value::new_integer(1)));
    expect_ok!(parse, "'true", quoted(Value::new_bool(true)));
    expect_ok!(parse, r#"'"2""#, quoted(Value::new_string("2")));
    expect_ok!(parse, "'#\\a", quoted(Value::new_char('a')));
    expect_ok!(parse, "'sym", quoted(Value::new_symbol("sym")));
    expect_ok!(parse, "'(1 2)", quoted(Value::new_list(&[Value::new_integer(1), Value::new_integer(2)])));
}
