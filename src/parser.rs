use nom::{digit, multispace};
use std::str;
use super::value::Value;

static UTF8_ERROR: &'static str = "File is no valid UTF8!";

named!(bool_<Value>, map!(
    alt!(
        tag!("true") |
        tag!("false")),
    |x|{ Value::new_bool(x == b"true") }));

named!(char_<Value>, chain!(
    tag!("'") ~
    c: take!(1) ~
    tag!("'") ,
    ||{ Value::new_char(c[0] as char) }));


named!(end_of_item,
    alt!(
        multispace |
        tag!(")")));

fn is_valid_in_ident(x: u8) -> bool {
    (x >= 0x3A && x <= 0x7E) ||
    (x >= 0x2A && x <= 0x2F) ||
    (x >= 0x23 && x <= 0x27) ||
    x == '!' as u8
}

named!(ident<Value>, chain!(
    peek!(none_of!("0123456789()")) ~
    x: take_while1!(is_valid_in_ident),
    || Value::new_ident((str::from_utf8(x).unwrap()).into())));

named!(integer<Value>,
    chain!(
        s: opt!(char!('-')) ~
        x: digit ,
        ||{
            let num: i64 = str::from_utf8(x).expect(UTF8_ERROR).parse().unwrap();
            if s.is_some() {
                Value::new_integer(-num)
            } else {
                Value::new_integer(num)
            }
        }));

named!(item<Value>,
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

named!(list_inner< Vec<Value> >,
    many0!(item));

named!(list<Value>, map!(
    delimited!(
        tag!("("),
        list_inner,
        tag!(")")),
    |x| Value::new_list(x)));

pub fn parse(input: &[u8]) -> Value {
    list(input).unwrap().1
}
