use std::usize;
use lalrpop_util::ParseError;
use ::value::Value;
use super::parse;
use super::lexer::Token;
use super::error::Error;
use ::string_interner::StringInterner;

const EOF: usize = usize::MAX;

fn whatever(x: &ParseError<usize, Token, Error>) -> usize {
    match x {
        &ParseError::InvalidToken{location: pos} => pos,
        &ParseError::UnrecognizedToken{expected: _, token: Some((pos, _, _))} => pos,
        &ParseError::User{error: Error::InvalidToken(_, end)} => end,
        &ParseError::User{error: Error::UnexpectedEofString(_)} => EOF,
        &ParseError::User{error: Error::UnexpectedEofChar(_)} => EOF,
        &ParseError::User{error: Error::NonAsciiChar(x)} => x,
        x => panic!("got: {:?}", x),
    }
}

macro_rules! expect_error {
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
        let interner = &mut StringInterner::new();
        let result = $parser(input, interner);

        match result {
            Err(ref err) => {
                if $use_pos && $token_fn(err) != $position {
                    panic!("for input: {:?}, got Err({:?}) at pos {}, expected pos: {}", input, err, $token_fn(err), $position);
                }
            },
            Ok(v) => panic!("expected error for string: '{}', got: {}", input, v.to_string(interner)),
        }
    );
}

macro_rules! expect_ok {
    // with rest
    ($parser:ident, $interner:expr, $input:expr, $expected:expr) => (
        let input = &*$input;
        let expected = $expected.to_string($interner);
        let result = $parser(input, $interner);
        match result {
            Ok(v) => {
                let res = v.to_string($interner);
                if res != expected {
                    panic!("parser ok, but input: {:?} got: Ok({:?}), expected: {}", input, res, expected);
                }
            },
            Err(e) => {
                panic!("input: {:?} got: Err({:?}), expected: {}", input, e, &expected);
            }
        }
    );
}

#[test]
fn bool() {
    let interner = &mut StringInterner::new();
    expect_ok!(parse, interner, "true", Value::new_bool(true));
    expect_ok!(parse, interner, "false", Value::new_bool(false));
    expect_ok!(parse, interner, "trude", Value::new_symbol(interner.intern("trude")));
    expect_ok!(parse, interner, "fale", Value::new_symbol(interner.intern("fale")));
}

#[test]
fn char() {
    let interner = &mut StringInterner::new();
    use std::char;
    use std::iter;
    let mut input: String = "#\\".into();
    let printlable_asci = ('!' as u32)..('~' as u32) + 1;
    for x in printlable_asci {
        let c = char::from_u32(x).expect(&format!("tried to create invalid char with: 0x{:X}", x));
        input.push(c);
        { expect_ok!(parse, interner, input, Value::new_char(c)); }
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
    expect_ok!(parse, interner, r"#\\s", Value::new_char(' '));
    expect_ok!(parse, interner, r"#\\t", Value::new_char('\t'));
    expect_ok!(parse, interner, r"#\\n", Value::new_char('\n'));
    expect_ok!(parse, interner, r"#\\", Value::new_char('\\'));

    expect_error!(parse, r"#\", EOF);
    expect_error!(parse, "#\\\0", 2);
}

#[test]
fn integer() {
    let interner = &mut StringInterner::new();
    expect_ok!(parse, interner, "007", Value::new_integer(7));
    expect_ok!(parse, interner, "-007", Value::new_integer(-7));
    expect_ok!(parse, interner, "123456789", Value::new_integer(123456789));
    expect_ok!(parse, interner, "-123456789", Value::new_integer(-123456789));

    expect_error!(parse, "123b456789", 3);
    expect_error!(parse, "123456789c", 9);
    expect_error!(parse, "00-7", 2);
    expect_ok!(parse, interner, "a123456789", Value::new_symbol(interner.intern("a123456789")));
    expect_ok!(parse, interner, "--7", Value::new_symbol(interner.intern("--7")));
}

#[test]
fn symbol() {
    let interner = &mut StringInterner::new();
    expect_ok!(parse, interner, "+", Value::new_symbol(interner.intern("+")));
    expect_ok!(parse, interner, "-", Value::new_symbol(interner.intern("-")));
    expect_ok!(parse, interner, "#", Value::new_symbol(interner.intern("#")));
    expect_ok!(parse, interner, "a1a", Value::new_symbol(interner.intern("a1a")));
    expect_ok!(parse, interner, "num->str", Value::new_symbol(interner.intern("num->str")));
    expect_ok!(parse, interner, "//", Value::new_symbol(interner.intern("//")));

    // error is at 1 bc lexer tries to lex integer
    expect_error!(parse, "1a", 1);
}

#[test]
fn string() {
    fn q(s: &str) -> String {
        format!("\"{}\"", s)
    }

    macro_rules! expect_str_ok {
        ($s:expr, $e:expr) => ({
            let s = q($s);
            let interner = &mut StringInterner::new();
            expect_ok!(parse, interner, s, Value::new_string($e));
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
    let interner = &mut StringInterner::new();
    let t = Value::new_bool(true);
    let f = Value::new_bool(false);
    let e = Value::empty_list();

    expect_ok!(parse, interner, "(true . false)", Value::new_pair(t.clone(), f.clone()));
    expect_ok!(parse, interner, "(true . (false . ()))", Value::new_pair(t, Value::new_pair(f.clone(), e.clone())));

    expect_error!(parse, "(1 .)", 4);
    expect_error!(parse, "(. 2)", 1);
    expect_error!(parse, "(1 . 2 3)", 7);
    expect_error!(parse, "(1 . 2 . 3)", 7);
    expect_error!(parse, "(1 2 . 3)", 5);
}

#[test]
fn list() {
    let interner = &mut StringInterner::new();
    expect_ok!(parse, interner, "()", Value::empty_list());
    expect_ok!(parse, interner, r#"(1 "2" (3 . 4))"#, Value::new_list(&vec![Value::new_integer(1), Value::new_string("2"), Value::new_pair(Value::new_integer(3), Value::new_integer(4))]));
    expect_ok!(parse, interner, "(() ())", Value::new_list(&vec![Value::empty_list(), Value::empty_list()]));

    expect_error!(parse, "(( ())");
}

#[test]
fn quote() {
    fn quoted(v: Value, interner: &mut StringInterner) -> Value {
        let quote_id = interner.intern("quote");
        Value::new_list(&[Value::new_symbol(quote_id), v])
    }
    let interner = &mut StringInterner::new();
    expect_ok!(parse, interner, "'()", quoted(Value::empty_list(), interner));
    expect_ok!(parse, interner, "'1", quoted(Value::new_integer(1), interner));
    expect_ok!(parse, interner, "'true", quoted(Value::new_bool(true), interner));
    expect_ok!(parse, interner, r#"'"2""#, quoted(Value::new_string("2"), interner));
    expect_ok!(parse, interner, "'#\\a", quoted(Value::new_char('a'), interner));
    expect_ok!(parse, interner, "'sym", quoted(Value::new_symbol(interner.intern("sym")), interner));
    expect_ok!(parse, interner, "'(1 2)", quoted(Value::new_list(&[Value::new_integer(1), Value::new_integer(2)]), interner));
}

#[test]
fn recur() {
    let interner = &mut StringInterner::new();

    expect_ok!(parse, interner, "recur", Value::new_symbol(interner.intern("recur")));
    expect_error!(parse, "(let () (recur) bla)");
}

#[test]
fn everything_together() {
    let interner = &mut StringInterner::new();
    let string = r#"("hi" my "NaMe" #\i #\s recur -42 # #\\n)"#;
    assert_eq!(parse(string, interner).unwrap().to_string(interner), r#"("hi" my "NaMe" #\i #\s recur -42 # #\\n)"#)
}
