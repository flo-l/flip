use nom::{digit, multispace, ErrorKind};
use std::str;
use super::value::Value;

static UTF8_ERROR: &'static str = "File is no valid UTF8!";

named!(bool_<&[u8], Value, ParserError>, fix_error!(ParserError, map!(
    alt!(
        tag!("true") |
        tag!("false")),
    |x|{ Value::new_bool(x == b"true") })));

named!(char_<&[u8], Value, ParserError>, fix_error!(ParserError, chain!(
    tag!("'") ~
    error!(ErrorKind::Custom(ParserError::MissingChar), not!(tag!("'"))) ~
    c: take!(1) ~
    error!(ErrorKind::Custom(ParserError::MultipleChars),
        fix_error!(ParserError, tag!("'"))) ,
    ||{ Value::new_char(c[0] as char) })));


named!(end_of_item<&[u8], &[u8], ParserError>,
    fix_error!(ParserError, alt!(
        multispace |
        tag!(")"))));

fn is_valid_in_ident(x: u8) -> bool {
    (x >= 0x3A && x <= 0x7E) ||
    (x >= 0x2A && x <= 0x2F) ||
    (x >= 0x23 && x <= 0x27) ||
    x == '!' as u8
}

named!(ident<&[u8], Value, ParserError>, fix_error!(ParserError, chain!(
    peek!(none_of!("0123456789()")) ~
    x: take_while1!(is_valid_in_ident),
    || Value::new_ident((str::from_utf8(x).unwrap())))));

named!(integer<&[u8], Value, ParserError>,
    fix_error!(ParserError, chain!(
        s: opt!(char!('-')) ~
        x: digit ,
        ||{
            let num: i64 = str::from_utf8(x).expect(UTF8_ERROR).parse().unwrap();
            if s.is_some() {
                Value::new_integer(-num)
            } else {
                Value::new_integer(num)
            }
        })));

named!(item<&[u8], Value, ParserError>,
    chain!(
        opt!(multispace) ~
        value: alt!(
            bool_ |
            char_ |
            integer |
            ident |
            list) ~
        peek!(end_of_item),
        || value));

named!(list_inner<Vec<Value> >,
    many0!(item));

named!(list<&[u8], Value, ParserError>,
    fix_error!(ParserError, map!(
        delimited!(
            tag!("("),
            list_inner,
            tag!(")")),
        |x| Value::new_list(x))));

pub fn parse(input: &[u8]) -> Value {
    list(input).unwrap().1
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ParserError {
    MultipleChars, // more than 1 character between '', eg. 'ab'
    MissingChar,   // no char between '', eg. ''
}

#[cfg(test)]
mod test {
    use super::{ParserError, bool_, char_, integer, ident};
    use nom::{IResult, Err, ErrorKind};
    use super::super::value::Value;

    macro_rules! expect_error {
        // with error kind
        ($parser:ident, $input:expr, $pos:expr, $errorkind:expr) => (
            let input = $input.as_bytes();
            let error_pos = &input[$pos..];
            let res = $parser(input);
            assert!(match res {
                IResult::Error(
                    Err::NodePosition(
                        ErrorKind::Custom(kind),
                        pos,
                        ref boxed_err)
                ) if (pos == error_pos && kind == $errorkind) => {
                    match &**boxed_err {
                        &Err::Position(_, pos) if pos == error_pos => true,
                        x => { println!("{:?}", x); false },
                    }
                },
                x => { println!("{:?}", x); false }
            });
        );
        // without error kind
        ($parser:ident, $input:expr, $pos:expr) => (
            let input = $input.as_bytes();
            let error_pos = &input[$pos..];
            assert!(match $parser(input) {
                IResult::Error(Err::Position(_, pos)) if pos == error_pos => true,
                x => { println!("{:?}", x); false },
            });
        )
    }

    macro_rules! expect_ok {
        // with rest
        ($parser:ident, $input:expr, $expected:expr, $rest:expr) => (
            let input = $input.as_bytes();
            let rest = $rest.as_bytes();
            assert_eq!(
                $parser(input),
                IResult::Done(rest, $expected)););

        // without rest
        ($parser:ident, $input:expr, $expected:expr) => (
            expect_ok!($parser, $input, $expected, "");
        )
    }

    #[test]
    fn bool() {
        expect_ok!(bool_, "true", Value::new_bool(true));
        expect_ok!(bool_, "false", Value::new_bool(false));
        expect_error!(bool_, "trude", 0);
        expect_error!(bool_, "fale", 0);
    }

    #[test]
    fn char() {
        use std::u8;
        for x in 0..127 {
            if x == '\'' as u8 { continue } // skip ''', which is invalid (tested below)
            let input = String::from_utf8(vec!['\'' as u8, x, '\'' as u8]).unwrap();
            expect_ok!(char_, input, Value::new_char(x as char));
        }
        expect_error!(char_, "'ab'", 2, ParserError::MultipleChars);
        expect_error!(char_, "''", 1, ParserError::MissingChar);
    }

    #[test]
    fn integer_() {
        expect_ok!(integer, "007", Value::new_integer(7));
        expect_ok!(integer, "-007", Value::new_integer(-7));
        expect_ok!(integer, "123456789", Value::new_integer(123456789));
        expect_ok!(integer, "-123456789", Value::new_integer(-123456789));
        expect_ok!(integer, "123b456789", Value::new_integer(123), "b456789");
        expect_ok!(integer, "123456789c", Value::new_integer(123456789), "c");
        expect_ok!(integer, "00-7", Value::new_integer(0), "-7");

        expect_error!(integer, "a123456789", 0);
        expect_error!(integer, "--7", 1);
    }

    #[test]
    fn ident_() {
        expect_ok!(ident, "+", Value::new_ident("+"));
        expect_ok!(ident, "a1a", Value::new_ident("a"), "1a");
        expect_ok!(ident, "num->str", Value::new_ident("num->str"));
        expect_ok!(ident, "//", Value::new_ident("//"));

        expect_error!(ident, "1a", 0);
    }
}
