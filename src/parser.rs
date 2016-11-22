use nom::{digit, multispace, eof, ErrorKind, IResult};
use std::str;
use std::fmt;
use super::value::Value;

static UTF8_ERROR: &'static str = "File is no valid UTF8!";

macro_rules! fix {
    ($i:expr, $it:ident) => (fix_error!($i, ParserError, $it));
    ($i:expr, $submac:ident!( $($args:tt)* )) => (
      fix_error!($i, ParserError, $submac!($($args)*))
    );
}

named!(bool_<&[u8], Value, ParserError>, fix!(map!(
    alt!(
        tag!("true") |
        tag!("false")),
    |x|{ Value::new_bool(x == b"true") })));

named!(char_<&[u8], Value, ParserError>, fix!(chain!(
    tag!("'") ~
    error!(ErrorKind::Custom(ParserError::MissingChar), not!(tag!("'"))) ~
    c: take!(1) ~
    error!(ErrorKind::Custom(ParserError::MultipleChars),
        fix!(tag!("'"))) ,
    ||{ Value::new_char(c[0] as char) })));


named!(end_of_item<&[u8], &[u8], ParserError>,
    fix!(alt!(
        multispace |
        eof |
        tag!(")"))));

fn is_valid_in_ident(x: u8) -> bool {
    (x >= 0x3A && x <= 0x7E) ||
    (x >= 0x2A && x <= 0x2F) ||
    (x >= 0x23 && x <= 0x27) ||
    x == '!' as u8
}

named!(ident<&[u8], Value, ParserError>, fix!(chain!(
    peek!(none_of!("0123456789().")) ~
    x: take_while1!(is_valid_in_ident),
    || Value::new_ident((str::from_utf8(x).unwrap())))));

named!(integer<&[u8], Value, ParserError>,
    fix!(chain!(
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

// eats everything until a " char is encountered
// recognizes escaped characters
fn eat_string_content(input: &[u8]) -> IResult<&[u8], String, ParserError> {
    fn save_str(input: &[u8], start: &mut usize, end: &mut usize, content: &mut String) {
        if start >= end { return }
        let utf8_str = match str::from_utf8(&input[*start..*end]) {
            Ok(s) => s,
            Err(x) => panic!("{:?}", x), // TODO: prettify this
        };
        content.push_str(utf8_str);
        *start = *end;
    }

    if input.len() == 0 {
        return IResult::Done(&input, String::new());
    }

    let mut end: usize = 0;
    let mut start: usize = 0;
    let mut content = String::new();
    while end < input.len() {
        println!("--------{}: {:?}", end, input[end] as char);
        if input[end] == '"' as u8 {
            break
        }
        println!("bla");
        if input[end] == '\\' as u8 {
            if (&input[end..]).len() <= 1 {
                break;
            }

            if input[end+1..].starts_with("n".as_bytes()) {
                save_str(input, &mut start, &mut end, &mut content);
                content.push('\n');
                start += 2;
                end += 2;
            } else if input[end+1..].starts_with("t".as_bytes()) {
                save_str(input, &mut start, &mut end, &mut content);
                content.push('\t');
                start += 2;
                end += 2;
            } else if input[end+1..].starts_with("\"".as_bytes()) {
                println!("save_str2({:?}, &mut {:?}, &mut {:?}, &mut {:?})", str::from_utf8(input), &mut start, &mut end, &mut content);
                save_str(input, &mut start, &mut end, &mut content);
                content.push('\"');
                start += 2;
                end += 2;
            } else if input[end+1..].starts_with("\\".as_bytes()) {
                save_str(input, &mut start, &mut end, &mut content);
                content.push('\\');
                start += 2;
                end += 2;
            } else {
                end += 1;
            }
        } else {
            end += 1;
        }
    }

    println!("save_str({:?}, &mut {:?}, &mut {:?}, &mut {:?})", str::from_utf8(input), &mut start, &mut end, &mut content);
    save_str(input, &mut start, &mut end, &mut content);
    println!("{:?}", str::from_utf8(&input[end..]));

    IResult::Done(&input[end..], content)
}

named!(string<&[u8], Value, ParserError>,
    chain!(
        fix!(tag!("\"")) ~
        x: eat_string_content ~
        fix!(tag!("\"")) ,
        || { Value::new_string(x) }
    ));

named!(item<&[u8], Value, ParserError>,
    complete!(chain!(
        opt!(multispace) ~
        quoted: opt!(tag!("'")) ~
        value: alt_complete!(
            bool_ |
            char_ |
            integer |
            string |
            ident |
            pair |
            list) ~
        peek!(end_of_item),
        || if quoted.is_some() {
            Value::new_pair(Value::new_ident("quote"), Value::new_pair(value, Value::empty_list()))
        } else { value })));

named!(pair<&[u8], Value, ParserError>,
    error!(ErrorKind::Custom(ParserError::InvalidPair), complete!(delimited!(
            fix!(tag!("(")),
            chain!(
                a: item ~
                fix!(multispace) ~
                fix!(tag!(".")) ~
                fix!(multispace) ~
                b: item ,
                || { Value::new_pair(a,b) }
            ),
            fix!(tag!(")"))))));

named!(list_inner<Vec<Value> >,
    many0!(item));

named!(list<&[u8], Value, ParserError>,
    alt!(
        fix!(map!(
            tag!("()") ,
            |_| Value::empty_list())) |
        fix!(map!(
            delimited!(
                tag!("("),
                list_inner,
                tag!(")")),
            |x: Vec<Value>| {
                let mut iter = x.into_iter().rev();
                let last = iter.next().unwrap(); // safe because list len must be >= 1
                iter.fold(Value::new_pair(last, Value::empty_list()), |prev_pair, value| Value::new_pair(value, prev_pair))
            }))
        ));

pub fn parse(input: &[u8]) -> Result<Value, String> {
    use nom::Err as NomErr;
    match item(input) {
        IResult::Done(remaining, value) => {
            if remaining.len() > 0 {
                Err(format!("error, expected EOF, found: {}", str::from_utf8(remaining).unwrap()))
            } else {
                Ok(value)
            }
        },
        IResult::Incomplete(_) => unimplemented!(), // because of complete
        IResult::Error(NomErr::Position(err, _)) => Err(create_error_string(err)),
        IResult::Error(NomErr::NodePosition(err, _, _)) => Err(create_error_string(err)),
        _ => unimplemented!(),
    }
}

fn create_error_string(err: ErrorKind<ParserError>) -> String {
    let msg = match err {
        ErrorKind::Custom(x) => format!("{}", x),
        x => format!("parser error {}", x.description()),
    };

    format!("error: {}", msg)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ParserError {
    MultipleChars, // more than 1 character between '', eg. 'ab'
    MissingChar,   // no char between '', eg. ''
    InvalidPair,   // eg. (1 2 . 3) or (1 . 2 3)
    InvalidItem,   // eg. 1 2
    NonUtf8String, // well..
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserError::MultipleChars => write!(f, "more than one character between ''"),
            ParserError::MissingChar => write!(f, "no character between ''"),
            ParserError::InvalidPair => write!(f, "invalid pair"),
            ParserError::InvalidItem => write!(f, "invalid item"),
            ParserError::NonUtf8String => write!(f, "string is no valid UTF8"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{ParserError, bool_, char_, integer, string, ident, pair, list, item, parse};
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
                ) if (pos == error_pos && kind == $errorkind) => { true },
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

    #[test]
    fn list_() {
        expect_ok!(list, "()", Value::empty_list());
        //TODO: add more tests for lists with content
    }

    #[test]
    fn pair_() {
        let t = Value::new_bool(true);
        let f = Value::new_bool(false);
        let e = Value::empty_list();

        expect_ok!(pair, "(true . false)", Value::new_pair(t.clone(), f.clone()));
        expect_ok!(pair, "(true . (false . ()))", Value::new_pair(t, Value::new_pair(f.clone(), e.clone())));

        expect_error!(pair, "(1 .)", 0, ParserError::InvalidPair);
        expect_error!(pair, "(. 2)", 0, ParserError::InvalidPair);
        expect_error!(pair, "(1 . 2 3)", 0, ParserError::InvalidPair);
        expect_error!(pair, "(1 . 2 . 3)", 0, ParserError::InvalidPair);
        expect_error!(pair, "(1 2 . 3)", 0, ParserError::InvalidPair);
    }

    #[test]
    fn string_() {
        fn q(s: &str) -> String { format!("\"{}\"", s) }
        fn uq(s: &str) -> &str { &s[1..s.len()-1] }

        macro_rules! expect_str_ok {
            ($s:expr, $e:expr) => ({
                let s = q($s);
                expect_ok!(string, s, Value::new_string($e));
            });

            ($s:expr) => (expect_str_ok!($s, $s));
        }

        expect_str_ok!("");
        expect_str_ok!("abc");
        expect_str_ok!("Hello, World!!");
        expect_str_ok!("\n");
        expect_str_ok!("\\n", "\n");
        expect_str_ok!("\t");
        expect_str_ok!("\\t", "\t");
        expect_str_ok!("\\\\", "\\");
        expect_str_ok!("Hi there: \\\" \\\\ \\n \\t", "Hi there: \" \\ \n \t");
    }

    //TODO
    #[test]
    fn parse_() {
        //expect_error!(parse, "1 1", 1, ParserError::InvalidItem);

    }
}
