#[derive(Debug)]
pub enum Error {
    // pos of invalid char
    NonAsciiChar(usize),

    // start..end
    InvalidEscape(usize, usize),
    InvalidToken(usize, usize),
    // start
    UnexpectedEofString(usize),
    UnexpectedEofChar(usize),

    // TODO: add more info
    RecurInNonTailPosition,
}
