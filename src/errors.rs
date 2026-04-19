use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum SyntaxError {
    // Lexer error types
    InvalidUnicodeSequence { invalid_content: Box<[u8]> },
    UnknownToken { unknown_content: Box<str> },
    InvalidIntLiteral,
    InvalidFloatLiteral,
    FloatLiteralIsInfinite,
    UnclosedStringLiteral,
    UnclosedMultiLineComment,

    // Parser error types
    UnexpectedToken { message: Box<str> },
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SyntaxError::*;

        match self {
            InvalidUnicodeSequence {
                invalid_content: erroneous_content,
            } => {
                write!(f, "invalid unicode sequence \"")?;
                for &b in erroneous_content.iter() {
                    write!(f, "{}", (b as char).escape_default())?;
                }
                write!(f, "\"")
            }
            UnknownToken { unknown_content } => write!(f, "unknown token {:?}", unknown_content),
            InvalidIntLiteral => write!(f, "integer literal was malformed or too large"),
            InvalidFloatLiteral => write!(f, "float literal was malformed"),
            FloatLiteralIsInfinite => write!(f, "float literal would be infinity"),
            UnclosedStringLiteral => write!(f, "string literal was not closed"),
            UnclosedMultiLineComment => write!(f, "multi-line comment was not closed"),
            UnexpectedToken { message } => write!(f, "unexpected token: {}", message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(
        SyntaxError::InvalidUnicodeSequence{
            invalid_content: Box::from([b'h', b'e', b'l', b'l', b'o', 0xff, 0xfe, 0xab, b'"', b'!', b'\\', b'n']),
        },
        "invalid unicode sequence \"hello\\u{ff}\\u{fe}\\u{ab}\\\"!\\\\n\""
        ; "invalid unicode sequence"
    )]
    #[test_case(
        SyntaxError::UnknownToken { unknown_content: Box::from("$foo") },
        "unknown token \"$foo\""
        ; "unknown token"
    )]
    #[test_case(
        SyntaxError::InvalidIntLiteral,
        "integer literal was malformed or too large"
        ; "invalid int literal"
    )]
    #[test_case(
        SyntaxError::InvalidFloatLiteral,
        "float literal was malformed"
        ; "invalid float literal"
    )]
    #[test_case(
        SyntaxError::FloatLiteralIsInfinite,
        "float literal would be infinity"
        ; "infinite float literal"
    )]
    #[test_case(
        SyntaxError::UnclosedStringLiteral,
        "string literal was not closed"
        ; "unclosed string literal"
    )]
    #[test_case(
        SyntaxError::UnclosedMultiLineComment,
        "multi-line comment was not closed"
        ; "unclosed multi-line comment"
    )]
    #[test_case(
        SyntaxError::UnexpectedToken { message: Box::from("expected foo but got bar") },
        "unexpected token: expected foo but got bar"
        ; "unexpected token"
    )]
    fn lexer_error_formats_correctly(error: SyntaxError, expected: &str) {
        // When
        let actual = format!("{}", error);

        // Then
        assert_eq!(actual, expected);
    }
}
