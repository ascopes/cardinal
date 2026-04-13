use crate::span::Spanned;
use std::fmt;

pub type LexerResult<'src, T> = Result<T, Spanned<LexerError<'src>>>;

#[derive(Clone, Debug, PartialEq)]
pub enum LexerError<'src> {
    InvalidUnicodeSequence(&'src [u8]),
    UnknownToken(&'src str),
    InvalidIntLiteral,
    InvalidFloatLiteral,
    FloatLiteralIsInfinite,
    UnclosedStringLiteral,
    UnclosedMultiLineComment,
}

impl<'src> fmt::Display for LexerError<'src> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LexerError::*;

        match self {
            InvalidUnicodeSequence(value) => {
                write!(f, "invalid unicode sequence \"")?;
                for &b in value.iter() {
                    write!(f, "{}", (b as char).escape_default())?;
                }
                write!(f, "\"")
            }
            UnknownToken(value) => write!(f, "unknown token {:?}", value),
            InvalidIntLiteral => write!(f, "integer literal was malformed or too large"),
            InvalidFloatLiteral => write!(f, "float literal was malformed"),
            FloatLiteralIsInfinite => write!(f, "float literal would be infinity"),
            UnclosedStringLiteral => write!(f, "string literal was not closed"),
            UnclosedMultiLineComment => write!(f, "multi-line comment was not closed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(
        LexerError::InvalidUnicodeSequence(&[b'h', b'e', b'l', b'l', b'o', 0xff, 0xfe, 0xab, b'"', b'!', b'\\', b'n']),
        "invalid unicode sequence \"hello\\u{ff}\\u{fe}\\u{ab}\\\"!\\\\n\""
        ; "invalid unicode sequence"
    )]
    #[test_case(    LexerError::UnknownToken("$foo"),                     "unknown token \"$foo\"" ; "unknown token")]
    #[test_case(       LexerError::InvalidIntLiteral, "integer literal was malformed or too large" ; "invalid int literal")]
    #[test_case(     LexerError::InvalidFloatLiteral,                "float literal was malformed" ; "invalid float literal")]
    #[test_case(  LexerError::FloatLiteralIsInfinite,            "float literal would be infinity" ; "infinite float literal")]
    #[test_case(   LexerError::UnclosedStringLiteral,              "string literal was not closed" ; "unclosed string literal")]
    #[test_case(LexerError::UnclosedMultiLineComment,          "multi-line comment was not closed" ; "unclosed multi-line comment")]
    fn lexer_error_formats_correctly(error: LexerError, expected: &str) {
        // When
        let actual = format!("{}", error);

        // Then
        assert_eq!(actual, expected);
    }
}
