use nom::{digit, space};
use std::str;
use std::fs::File;
use std::io::Read;

static UTF8_ERROR: &'static str = "File is no valid UTF8!";

#[derive(Debug)]
pub enum IR {
    Plus,
    Integer(i64),
    List(Vec<IR>),
}

named!(plus<IR>, map!(
    tag!("+"), |_| IR::Plus));

named!(integer<IR>,
    chain!(
        s: opt!(tag!("-")) ~
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
        ir: alt!(
            plus |
            integer) ~
        alt!(
            space |
            not!(tag!(")"))),
        || ir));


named!(list_inner< Vec<IR> >,
    many0!(item));

named!(list<IR>, map!(
    delimited!(
        char!('('),
        list_inner,
        char!(')')),
    |x| IR::List(x)));

pub fn parse(file: File) -> IR {
    let bytes: Vec<u8> = file.bytes().filter_map(Result::ok).collect();
    list(&bytes).unwrap().1
}
