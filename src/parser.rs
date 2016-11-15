use nom::{digit, multispace};
use std::str;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use super::ir::IR;

static UTF8_ERROR: &'static str = "File is no valid UTF8!";

named!(bool_<IR>, map!(
    alt!(
        tag!("true") |
        tag!("false")),
    |x|{ IR::Bool(x == b"true") }));

named!(char_<IR>, chain!(
    tag!("'") ~
    c: take!(1) ~
    tag!("'") ,
    ||{ IR::Char(c[0] as char) }));


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

named!(ident<IR>, chain!(
    peek!(none_of!("0123456789()")) ~
    x: take_while1!(is_valid_in_ident),
    || IR::Ident(Rc::new((str::from_utf8(x).unwrap()).into()))));

named!(integer<IR>,
    chain!(
        s: opt!(char!('-')) ~
        x: digit ,
        ||{
            let num: i64 = str::from_utf8(x).expect(UTF8_ERROR).parse().unwrap();
            if s.is_some() {
                IR::Integer(-num)
            } else {
                IR::Integer(num)
            }
        }));

named!(item<IR>,
    chain!(
        opt!(multispace) ~
        ir: alt!(
            bool_ |
            char_ |
            integer |
            ident |
            list) ~
        peek!(end_of_item),
        || ir));

named!(list_inner< Vec<IR> >,
    many0!(item));

named!(list<IR>, map!(
    delimited!(
        tag!("("),
        list_inner,
        tag!(")")),
    |x| IR::List(Rc::new(x))));

pub fn parse(file: File) -> IR {
    let bytes: Vec<u8> = file.bytes().filter_map(Result::ok).collect();
    list(&bytes).unwrap().1
}
