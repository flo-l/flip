use std::str::CharIndices;
use std::ascii::AsciiExt;
use std::iter::Peekable;

use super::error::Error;

// use ascii end of transmission as EOF
const EOF: char = 0x4 as char;

macro_rules! my_try {
    ($e:expr) => ({
        match $e {
            Ok(x) => x,
            Err(x) => return Some(Err(x)),
        }
    });
}

#[derive(Clone, Copy, Debug)]
pub enum Token<'input> {
    OpenParen,
    ClosingParen,
    Dot,
    QuoteTick,
    WhiteSpace,
    Char(char),
    Integer(i64),
    String(&'input str),
    Symbol(&'input str),
    True,
    False,
    Begin,
    Define,
    If,
    Let,
    Loop,
    Lambda,
    Recur,
    Quote,
}

// Tokenzer state
#[derive(Debug, Clone, Copy)]
enum State<'input> {
    // start
    NewToken,
    Finished(Spanned<Token<'input>>),
    // start
    EatInteger(usize),
    WhiteSpace(usize),
    Minus(usize),
    Symbol(usize),
    StringStart(usize),
    StringBackslash(usize),
    Pound(usize),
    CharBegin(usize),
    EscapedChar(usize),
    FinishedChar(usize, char),
}
use self::State::*;

#[derive(Debug)]
pub struct Tokenizer<'input> {
    text: &'input str,
    chars: Peekable<CharIndices<'input>>,
    state: State<'input>,
}

pub type Spanned<T> = (usize, T, usize);

impl<'input> Tokenizer<'input> {
    pub fn new(text: &'input str) -> Tokenizer<'input> {
        Tokenizer {
            text: text,
            chars: text.char_indices().peekable(),
            state: NewToken,
        }
    }

    fn next_char(&mut self) -> Option<(usize, char)> {
        self.chars.next()
    }

    fn peek_next(&mut self) -> Option<&(usize, char)> {
        self.chars.peek()
    }
}

impl<'input> Iterator for Tokenizer<'input> {
    type Item = Result<Spanned<Token<'input>>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next;
            match self.peek_next() {
                Some(&(pos, c)) => next = (self.state, pos, c),
                // check if we still need something
                None => next = (self.state, self.text.len(), EOF),
            }

            self.state = match next {
                // pos + 1 is safe, because all these chars have width 1 byte
                (NewToken, pos, '(') => { self.next_char(); Finished((pos, Token::OpenParen, pos+1)) },
                (NewToken, pos, ')') => { self.next_char(); Finished((pos, Token::ClosingParen, pos+1)) },
                (NewToken, pos, '\'') => { self.next_char(); Finished((pos, Token::QuoteTick, pos+1)) },
                (NewToken, pos, '.') => { self.next_char(); Finished((pos, Token::Dot, pos+1)) },
                (NewToken, pos, '-') => Minus(pos),
                (NewToken, pos, '"') => StringStart(pos),
                (NewToken, pos, '#') => Pound(pos),
                (NewToken, _, EOF) => return None,
                (NewToken, pos, c) if whitespace(c) => WhiteSpace(pos),
                (NewToken, pos, c) if numeric(c) => EatInteger(pos),
                (NewToken, pos, _) => Symbol(pos),

                // minus special case
                (Minus(start), _, c) if numeric(c) => EatInteger(start),
                (Minus(start), end, c) if end_of_item(c) => Finished((start, Token::Symbol(&self.text[start..end]), end)),
                (Minus(start), _, _) => Symbol(start),

                // whitespace
                (WhiteSpace(pos), _, c) if whitespace(c) => WhiteSpace(pos),
                (WhiteSpace(start), end, _) => Finished((start, Token::WhiteSpace, end)),

                // chars
                (Pound(pos), _, '\\') => CharBegin(pos),
                (Pound(start), end, c) if end_of_item(c) => Finished((start, Token::Symbol(&self.text[start..end]), end)),
                (Pound(pos), _, _) => Symbol(pos),

                (CharBegin(pos), _, '\\') => EscapedChar(pos),
                (CharBegin(_), _, EOF) => return Some(Err(Error::UnexpectedEofChar(self.text.len()))),
                (CharBegin(pos), _, c) if printable_char(c) => FinishedChar(pos, c),
                (CharBegin(_), pos, _) => return Some(Err(Error::NonAsciiChar(pos))),

                (EscapedChar(pos), _, c) if unescape_char(c).is_some() => FinishedChar(pos, unescape_char(c).unwrap()),
                (EscapedChar(start), end, EOF) => Finished((start, Token::Char('\\'), end)),
                (EscapedChar(start), end, _) => return Some(Err(Error::InvalidEscape(start+2, next_char(&self.text, end)))), // +2 bc #\ is 2 bytes

                (FinishedChar(start, x), end, c) if end_of_item(c) => Finished((start, Token::Char(x), end)),
                (FinishedChar(_, _), end, _) => return Some(Err(Error::InvalidToken(end, end))),

                // integers
                (EatInteger(start), _, c) if numeric(c) => EatInteger(start),
                (EatInteger(start), pos, c) if end_of_item(c) =>
                    // safe because we checked that text[start..pos] is a valid number
                    Finished((start, Token::Integer(self.text[start..pos].parse().unwrap()), pos)),
                (EatInteger(_), pos, _) => return Some(Err(Error::InvalidToken(pos, pos))),

                // strings
                (StringStart(start), end, '"') => {
                    self.next_char(); // bump
                    let string = &self.text[start+1..end];
                    for (i, c) in string.char_indices() {
                        if !c.is_ascii() {
                            return Some(Err(Error::NonAsciiChar(i+1))) // +1 bc start+1 above
                        }
                    }
                    Finished((start, Token::String(string), end+1))
                },
                (StringStart(_), _, EOF) => return Some(Err(Error::UnexpectedEofString(self.text.len()))),
                (StringStart(start), _, '\\') => StringBackslash(start),
                (StringStart(start), _, _) => StringStart(start),
                (StringBackslash(start), _, '"') => StringStart(start),
                (StringBackslash(start), _, c) if unescape_char(c).is_some() => StringStart(start),
                (StringBackslash(_), _, EOF) => return Some(Err(Error::UnexpectedEofString(self.text.len()))),
                (StringBackslash(_), pos, _) => return Some(Err(Error::InvalidEscape(pos-1, next_char(&self.text, pos)))), // -1 bc \ is 1 byte

                // symbols
                (Symbol(start), end, c) if end_of_item(c) => {
                    let token = match &self.text[start..end] {
                        "begin" => Token::Begin,
                        "define" => Token::Define,
                        "if" => Token::If,
                        "let" => Token::Let,
                        "loop" => Token::Loop,
                        "lambda" => Token::Lambda,
                        "recur" => Token::Recur,
                        "quote" => Token::Quote,
                        "true" => Token::True,
                        "false" => Token::False,
                        x => Token::Symbol(x),
                    };

                    Finished((start, token, end))
                },
                (Symbol(start), _, _) => Symbol(start),
                (Finished(_), _, _) => unreachable!(),
            };

            match self.state {
                Finished(token) => {
                    self.state = NewToken;
                    return Some(Ok(token))
                },
                _ if next.2 == EOF => self.state = NewToken, // fuse iterator
                _  => { self.next_char(); }, // bump
            }
        }
    }
}

fn whitespace(x: char) -> bool {
    x == ' ' ||
    x == '\n' ||
    x == '\t'
}

fn end_of_item(x: char) -> bool {
    whitespace(x) ||
    x == ')' ||
    x == EOF
}

pub fn escape_char(x: char) -> Option<char> {
    match x {
        '\n' => Some('n'),
        ' ' => Some('s'),
        '\t' => Some('t'),
        '\\' => Some('\\'),
        _ => None,
    }
}

fn unescape_char(x: char) -> Option<char> {
    match x {
        'n' => Some('\n'),
        's' => Some(' '),
        't' => Some('\t'),
        '\\' => Some('\\'),
        _ => None,
    }
}

fn next_char(text: &str, pos: usize) -> usize {
    pos + char::len_utf8(text.as_bytes()[pos] as char)
}

fn numeric(x: char) -> bool {
    '0' <= x && x <= '9'
}

fn printable_char(x: char) -> bool {
    x >= '!' &&
    x <= '~'
}

#[allow(dead_code)]
pub fn unescape_string(input: &str) -> String {
    let mut chars = input.chars();
    let mut s = String::with_capacity(input.chars().count());

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                match next {
                    'n' => s.push('\n'),
                    't' => s.push('\t'),
                    '"' => s.push('"'),
                    's' => s.push(' '),
                    '\\' => s.push('\\'),
                    x => {
                        s.push('\\');
                        s.push(x);
                    }
                }
                continue;
            }
        }
        s.push(c);
    }
    s
}
