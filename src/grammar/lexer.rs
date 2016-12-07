use std::str::CharIndices;
use std::ascii::AsciiExt;
use std::iter::Peekable;
use std::ops::Range;

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
    Quote,
    WhiteSpace,
    Char(char),
    Integer(i64),
    String(&'input str),
    Symbol(&'input str),
}

#[derive(Debug)]
pub struct Tokenizer<'input> {
    text: &'input str,
    chars: Peekable<CharIndices<'input>>,
}

pub type Spanned<T> = (usize, T, usize);

#[derive(Debug)]
pub enum Error {
    InvalidCharacter(char),
    UnexpectedToken(usize),
    UnexpectedEof,
}

impl<'input> Tokenizer<'input> {
    pub fn new(text: &'input str) -> Tokenizer<'input> {
        Tokenizer {
            text: text,
            chars: text.char_indices().peekable(),
        }
    }

    fn next_char(&mut self) -> Option<(usize, char)> {
        self.chars.next()
    }

    fn peek_next(&mut self) -> Option<&(usize, char)> {
        self.chars.peek()
    }

    // consumes all tokens until f returns true.
    // leaves the token for which f returned true in self.chars()
    fn eat_until<F: Fn(char) -> bool>(&mut self, start: usize, f: F) -> &'input str {
        match self.eat_until_strict(start, f) {
            Ok(x) => x,
            Err(_) => &self.text[start..],
        }
    }

    // same as eat_until, but returns an error if eof was encountered before f returned true
    fn eat_until_strict<F: Fn(char) -> bool>(&mut self, start: usize, f: F) -> Result<&'input str, Error> {
        while let Some(&(pos, c)) = self.peek_next() {
            if f(c) {
                return Ok(&self.text[start..pos]);
            } else {
                self.next_char(); //bump
            }
        }
        Err(Error::UnexpectedEof)
    }

    fn token_at(at: Range<usize>, tok: Token) -> Option<Result<Spanned<Token>, Error>> {
        Some(Ok((at.start, tok, at.end)))
    }

    fn parse_char(&mut self, start: usize) -> Option<Result<Spanned<Token<'input>>, Error>> {
        let s = self.eat_until(start, |c| end_of_item(c) && c != ')');

        if s.len() <= 2 {
            Tokenizer::token_at(start..start+s.len(), Token::Symbol(s))
        } else if s.len() == 3 {
            let c = s.chars().skip(2).next().unwrap();
            if valid_char(c) {
                Tokenizer::token_at(start..start+4, Token::Char(c))
            } else {
                Some(Err(Error::UnexpectedToken(start+2)))
            }
        } else if s.len() == 4 {
            let token = match s {
                r"#\\n" => Token::Char('\n'),
                r"#\\t" => Token::Char('\t'),
                r"#\\s" => Token::Char(' '),
                _ => return Some(Err(Error::UnexpectedToken(start+2))),
            };
            Tokenizer::token_at(start+2..start+4, token)
        } else {
            Some(Err(Error::UnexpectedToken(start+2)))
        }
    }

    fn parse_integer(&mut self, start: usize) -> Option<Result<Spanned<Token<'input>>, Error>> {
        let int_slice = self.eat_until(start, end_of_item);
        for (pos, c) in int_slice.char_indices() {
            if !numeric(c) {
                if pos == 0 && c == '-' { continue }
                return Some(Err(Error::UnexpectedToken(pos)));
            }
        }
        let num: i64 = int_slice.parse().unwrap();
        Tokenizer::token_at(start..start+int_slice.len(), Token::Integer(num))
    }

    fn parse_symbol(&mut self, start: usize) -> Option<Result<Spanned<Token<'input>>, Error>> {
        let symbol = self.eat_until(start, end_of_item);
        Tokenizer::token_at(start..start+symbol.len(), Token::Symbol(symbol))
    }

    fn parse_string(&mut self, start: usize) -> Option<Result<Spanned<Token<'input>>, Error>> {
        let mut str_len = 0;
        loop {
            match self.eat_until_strict(start + str_len, |c| c == '"') {
                Ok(substr) => {
                    self.next_char(); // bump so that '"' is skipped in next iteration/token

                    let mut rev_iter = substr.chars().rev().map(|x| x == '\\');
                    let last_was_slash = rev_iter.next().unwrap_or(false);
                    let second_last_was_slash = rev_iter.next().unwrap_or(false);

                    if last_was_slash && !second_last_was_slash {
                        str_len += substr.len() + 1; // + 1: include '"'
                    } else {
                        str_len += substr.len();
                        let s = &self.text[start..start+str_len];
                        // check for ascii only strings (for now)
                        for c in s.chars() {
                            if !c.is_ascii() {
                                return Some(Err(Error::InvalidCharacter(c)));
                            }
                        }
                        return Tokenizer::token_at(start..start+str_len, Token::String(s));
                    }
                }
                Err(x) => return Some(Err(x)),
            };
        }
    }
}

impl<'input> Iterator for Tokenizer<'input> {
    type Item = Result<Spanned<Token<'input>>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.next_char() {
                Some((s, '(')) => return Tokenizer::token_at(s..s+1, Token::OpenParen),
                Some((s, ')')) => return Tokenizer::token_at(s..s+1, Token::ClosingParen),
                Some((s, '\'')) => return Tokenizer::token_at(s..s+1, Token::Quote),
                Some((s, '.')) => return Tokenizer::token_at(s..s+1, Token::Dot),
                Some((s, '#')) => return self.parse_char(s),
                // special case '-'
                Some((start, '-')) => {
                    if let Some(&(_, c)) = self.peek_next() {
                        // we got a number
                        if numeric(c) {
                            return self.parse_integer(start);
                        }
                    }
                    return self.parse_symbol(start);
                },
                // number
                Some((start, x)) if numeric(x) => return self.parse_integer(start),
                // whitespace
                Some((start, x)) if whitespace(x) => {
                    let len = self.eat_until(start, |c| !whitespace(c)).len();
                    return Tokenizer::token_at(start..len, Token::WhiteSpace);
                },
                // string, + 1 bc we don't need the '"' in string content
                Some((start, '"')) => return self.parse_string(start+1),
                // symbol
                Some((start, _)) => return self.parse_symbol(start),
                None => return None,
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
    x == ')'
}

fn numeric(x: char) -> bool {
    '0' <= x && x <= '9'
}

fn valid_char(x: char) -> bool {
    x >= '!' &&
    x <= '~'
}
