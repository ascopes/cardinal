use crate::ast::expr::{Expr, StrLitExpr};
use crate::errors::SyntaxError;
use crate::parser::base::ParserResult;
use crate::spans::{Span, Spanned};
use std::str::CharIndices;

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
/// - `\uXXXX` where `XXXX` is a hexadecimal number translates to that codepoint in the UTF-8
///   plane.
/// - The string contents are expected to be UTF-8 encoded sequences. Anything else is deemed
///   garbage.
pub(super) fn parse_str_lit(raw: &str, span: Span) -> ParserResult<Expr> {
    // Throw away the open and close quotes.
    let inner = &raw[1..raw.len() - 1];
    let mut chars = inner.char_indices();
    let mut parsed = String::with_capacity(inner.len());

    while let Some(pair) = chars.next() {
        match pair {
            // index + 1 as we skipped the open quote
            (index, '\\') => {
                let char = parse_str_lit_escape(&mut chars, span, index + 1)?;
                parsed.push(char);
            }
            (_, char) => parsed.push(char),
        }
    }

    Ok(Spanned::new(
        Expr::Str(Box::new(StrLitExpr {
            value: parsed.into_boxed_str(),
        })),
        span,
    ))
}

fn parse_str_lit_escape(
    chars: &mut CharIndices,
    span: Span,
    index: usize,
) -> Result<char, Spanned<SyntaxError>> {
    match chars.next() {
        Some((_, '\\')) => Ok('\\'),
        Some((_, '"')) => Ok('"'),
        Some((_, 'n')) => Ok('\n'),
        Some((_, 'r')) => Ok('\r'),
        Some((_, 't')) => Ok('\t'),
        Some((_, 'u')) => parse_str_lit_unicode_codepoint(chars, span, index),
        Some((_, c)) => Err(Spanned::new(
            SyntaxError::InvalidStringEscapeSequence {
                sequence: format!("\\{}", c).into_boxed_str(),
            },
            Span::new(span.start() + index, span.start() + index + 2),
        )),
        None => Err(Spanned::new(
            SyntaxError::UnexpectedEndOfString,
            Span::new(span.start() + index, span.start() + index + 2),
        )),
    }
}

/// Parse the four hex digits after `\u` into a UTF-8 char.
fn parse_str_lit_unicode_codepoint(
    chars: &mut CharIndices,
    span: Span,
    index: usize,
) -> Result<char, Spanned<SyntaxError>> {
    let raw_codepoint = chars.take(4).map(|(_, c)| c).collect::<String>();

    // less than 4 characters is not valid.
    // e.g. if we reached end of string early.
    if raw_codepoint.len() != 4 {
        return Err(Spanned::new(
            SyntaxError::InvalidStringEscapeSequence {
                sequence: format!("\\u{}", raw_codepoint).into_boxed_str(),
            },
            Span::new(span.start() + index, span.start() + index + 6),
        ));
    }

    u32::from_str_radix(&raw_codepoint, 16)
        .ok()
        .and_then(|codepoint| char::from_u32(codepoint))
        .ok_or_else(|| {
            Spanned::new(
                SyntaxError::InvalidStringEscapeSequence {
                    sequence: format!("\\u{}", raw_codepoint).into_boxed_str(),
                },
                Span::new(span.start() + index, span.start() + index + 6),
            )
        })
}
