use ::lalrpop_util::ParseError;
use super::lexer::{Token, Error};
use std::iter;

pub fn create_error_message(input: &str, err: &ParseError<usize, Token, Error>) -> String {
    match err {
        &ParseError::InvalidToken { location } => invalid_token(input, location),
        &ParseError::UnrecognizedToken {
            token: Some((left, _token, right)),
            expected: _,
        } => unrecognized_token(input, left, right),
        &ParseError::UnrecognizedToken {
            token: None,
            expected: _,
        } => unexpected_eof(&input),
        &ParseError::ExtraToken {
            token: (left, _token, right),
        } => extra_token(input, left, right),
        &ParseError::User {
            ref error,
        } => tokenizer_error(input, error),
    }
}

fn print_line_with_pos(input: &str, start: usize, mut end: usize) -> String {
    // protect against start == end
    if start >= end { end = start + 1; }

    let mut line_no = 0;
    let mut line_start = 0;
    let mut line_end = input.len();
    let mut finished = false;
    for (pos, c) in input.char_indices() {
        if pos >= start { finished = true; }
        if c == '\n' {
            if finished {
                line_end = pos;
                break;
            }
            line_start = pos+1;
            line_no += 1;
        }
    }

    let span: String = iter::repeat('^').take(end-start).collect();

    format!(
"{} | {}
{:ws_pipe$} | {:ws_err$}{} ",
        line_no,
        &input[line_start..line_end],
        "", "", span,
        ws_pipe = line_no.to_string().len(),
        ws_err = input[line_start..start].chars().count(),
    )
}

fn invalid_token(input: &str, start: usize) -> String {
    concat_strings(&[
        print_line_with_pos(input, start, start),
        print_error_msg("invalid token"),
    ])
}

fn unrecognized_token(input: &str, start: usize, end: usize) -> String {
    concat_strings(&[
        print_line_with_pos(input, start, end),
        print_error_msg("unrecognized token"),
    ])
}

fn extra_token(input: &str, start: usize, end: usize) -> String {
    concat_strings(&[
        print_line_with_pos(input, start, end),
        print_error_msg("extra token"),
    ])
}

fn unexpected_eof(input: &str) -> String {
    //check if there are imbalanced parens
    let missing_parens = input.chars().fold(0i32, |x, c| {
        match c {
            '(' => x + 1,
            ')' => x - 1,
            _ => x
        }
    });

    let parens_hint = if missing_parens > 0 {
        let parens: String = iter::repeat(')').take(missing_parens as usize).collect();
        format!("unclosed parens, maybe you're missing '{}'?", parens)
    } else {
        "".into()
    };

    concat_strings(&[
        print_line_with_pos(input, input.len(), input.len()),
        print_error_msg("unexpected EOF"),
        print_hint_msg(&parens_hint),
    ])
}

fn tokenizer_error(input: &str, err: &Error) -> String {
    let strs = match err {
        &Error::InvalidCharacter(pos) => {
            // find char at position
            let c = input[pos..].chars().next().expect("internal error");
            vec![
                print_line_with_pos(input, pos, pos),
                print_error_msg(&format!("invalid character: {} is not ASCII", c)),
                "".into(),
            ]
        },
        &Error::UnexpectedEof => vec![
            print_line_with_pos(input, input.len(), input.len()),
            print_error_msg(&format!("unexpected EOF\n")),
            print_hint_msg("missing closing \", did you forget to terminate a string literal?")
        ],
        &Error::UnexpectedToken(pos) => {
            // find char at position
            let c = input[pos..].chars().next().expect("internal error");
            vec![
                print_error_msg(&format!("unexpected token: '{}'", c)),
            ]
        },
    };

    concat_strings(&strs)
}

fn print_error_msg(msg: &str) -> String {
    format!("error: {}", msg)
}

fn print_hint_msg(hint: &str) -> String {
    if hint.len() > 0 {
        format!("\nhint: {}", hint)
    } else {
        "".into()
    }
}

fn concat_strings(strings: &[String]) -> String {
    strings.into_iter().fold(String::new(), |mut res, s| {res.push_str(s); res})
}
