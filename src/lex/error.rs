use std::num::{ParseFloatError, ParseIntError};

#[derive(Clone, Debug)]
pub enum LexerError<'src> {
    UnknownToken(&'src str),
    InvalidUnicodeCharacter(&'src [u8]),
    IntParseError(ParseIntError),
    FloatParseError(ParseFloatError),
    UnclosedStringLiteral,
    UnclosedMultilineComment,
}
