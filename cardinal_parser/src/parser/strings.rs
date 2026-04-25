use crate::ast::expr::{Expr, StrLitExpr};
use crate::errors::SyntaxError;
use crate::parser::base::ParserResult;
use crate::spans::{Span, Spanned};

/// We fully parse string literals here.
///
/// Any escape sequences are validated and consumed properly, we throw away the wrapping quotes,
/// and generally validate the string. We do this here rather than in the lexer such that we
/// can potentially break out into supporting string interpolation at a later point in time.
///
/// Transformation notes:
///
/// - `\n` translates to an ASCII linefeed 0x0A.
/// - `\r` translates to an ASCII carriage return 0x0D.
/// - `\t` translates to an ASCII horizontal tab 0x09.
/// - `\\` translates to a single backslash `\`.
/// - `\"` translates to a literal double quote `"`.
/// - `\u{XXXX}` where `XXXX` is a hexadecimal number translates to that codepoint in the UTF-8
///   plane.
/// - The string contents are expected to be UTF-8 encoded sequences. Anything else is deemed
///   garbage.
pub(super) fn parse_str_lit(raw: &str, span: Span) -> ParserResult<Expr> {
    let inner = &raw[1..raw.len() - 1];
    // Throw away the open and close quotes.
    let mut chars = inner.char_indices().peekable();
    let mut parsed = String::with_capacity(inner.len());

    while let Some((index, char)) = chars.next() {
        match char {
            '\\' => match chars.next() {
                Some((_, '\\')) => parsed.push('\\'),
                Some((_, 'n')) => parsed.push('n'),
                Some((_, 'r')) => parsed.push('r'),
                Some((_, 't')) => parsed.push('t'),
                Some((_, 'u')) => unimplemented!("unicode escape handling is not implemented yet"),
                Some((_, c)) => {
                    return Err(Spanned::new(
                        SyntaxError::UnknownStringEscapeSequence {
                            sequence: format!("\\{}", c).into_boxed_str(),
                        },
                        Span::new(span.start() + index + 1, span.start() + index + 3),
                    ));
                }
                None => {
                    return Err(Spanned::new(
                        SyntaxError::UnexpectedEndOfString,
                        Span::new(span.start() + index + 1, span.start() + index + 3),
                    ));
                }
            },
            _ => parsed.push(char),
        }
    }

    Ok(Spanned::new(
        Expr::Str(Box::new(StrLitExpr {
            value: parsed.into_boxed_str()
        })),
        span
    ))
}
