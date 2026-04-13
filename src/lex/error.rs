use crate::span::Spanned;
use std::num::{ParseFloatError, ParseIntError};

pub type LexerResult<'src, T> = Result<T, Spanned<LexerError<'src>>>;

#[derive(Clone, Debug, PartialEq)]
pub enum LexerError<'src> {
    UnknownToken(&'src str),
    InvalidUnicodeSequence(&'src [u8]),
    InvalidIntLiteral,
    InvalidFloatLiteral,
    FloatLiteralIsInfinite,
    UnclosedStringLiteral,
    UnclosedMultilineComment,
}
