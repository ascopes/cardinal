use std::num::{ParseFloatError, ParseIntError};

#[derive(Clone, Debug, PartialEq)]
pub enum LexerError<'src> {
    UnknownToken(&'src str),
    InvalidUnicodeSequence(&'src [u8]),
    IntParseError(String),
    FloatParseError(String),
    UnclosedStringLiteral,
    UnclosedMultilineComment,
}
