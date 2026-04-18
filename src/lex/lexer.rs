use crate::lex::error::{LexerError, LexerResult};
use crate::lex::token::{Token, TokenKind};
use crate::span::{Span, Spanned};
use std::str::FromStr;

type LexerTokenResult<'src> = LexerResult<'src, Spanned<Token<'src>>>;

/// Basic lexer that parses a UTF-8 encoded source file provided as a byte array.
///
/// Each new line is recorded such that token spans can be mapped back to the offset
/// of the start of the current line of code. This allows reconstructing positional
/// details later without copying that information into every single token.
#[derive(Debug)]
pub struct Lexer<'src> {
    src: &'src [u8],

    offset: usize,

    // Store the offsets for the start of each line as we hit it. The parser can query this
    // later to find the exact line number when producing an error message, given the offset
    // in a span.
    line_starts: Vec<usize>,
}

impl<'src> Lexer<'src> {
    pub fn new(src: &'src [u8]) -> Self {
        Self {
            src,
            offset: 0,
            line_starts: vec![0],
        }
    }

    /// Determine the line number that the given span corresponds to. Error reporters
    /// can use this to work out the exact position of an erroneous token later.
    ///
    /// Returns a tuple of the line number and the starting offset of that line.
    pub fn get_line_number_of(&self, span: Span) -> (usize, usize) {
        let line_number = self
            .line_starts
            .partition_point(|&start| start <= span.start())
            - 1;
        (line_number, self.line_starts[line_number])
    }

    /// Get the next token.
    pub fn next_token(&mut self) -> LexerTokenResult<'src> {
        self.skip_whitespace();

        let start = self.offset;

        match self.peek(0) {
            None => self.scan_eof(),
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'_') => self.scan_ident(),
            Some(b'0'..=b'9') => self.scan_num_lit(),
            Some(b'.') if matches!(self.peek(1), Some(b'0'..=b'9')) => self.scan_num_lit(),
            Some(b'"') => self.scan_str_lit(),
            Some(b'+') => match self.peek(1) {
                Some(b'=') => self.advance_and_emit(start, 2, TokenKind::AddAssign),
                _ => self.advance_and_emit(start, 1, TokenKind::Add),
            },
            Some(b'-') => match self.peek(1) {
                Some(b'=') => self.advance_and_emit(start, 2, TokenKind::SubAssign),
                _ => self.advance_and_emit(start, 1, TokenKind::Sub),
            },
            Some(b'*') => match self.peek(1) {
                Some(b'*') => match self.peek(2) {
                    Some(b'=') => self.advance_and_emit(start, 3, TokenKind::PowAssign),
                    _ => self.advance_and_emit(start, 2, TokenKind::Pow),
                },
                Some(b'=') => self.advance_and_emit(start, 2, TokenKind::MulAssign),
                _ => self.advance_and_emit(start, 1, TokenKind::Mul),
            },
            Some(b'/') => match self.peek(1) {
                Some(b'/') => self.scan_single_line_comment(),
                Some(b'*') => self.scan_multi_line_comment(),
                Some(b'=') => self.advance_and_emit(start, 2, TokenKind::DivAssign),
                _ => self.advance_and_emit(start, 1, TokenKind::Div),
            },
            Some(b'%') => match self.peek(1) {
                Some(b'=') => self.advance_and_emit(start, 2, TokenKind::ModAssign),
                _ => self.advance_and_emit(start, 1, TokenKind::Mod),
            },
            Some(b'~') => self.advance_and_emit(start, 1, TokenKind::BitNot),
            Some(b'&') => match self.peek(1) {
                Some(b'&') => self.advance_and_emit(start, 2, TokenKind::BoolAnd),
                Some(b'=') => self.advance_and_emit(start, 2, TokenKind::BitAndAssign),
                _ => self.advance_and_emit(start, 1, TokenKind::BitAnd),
            },
            Some(b'|') => match self.peek(1) {
                Some(b'|') => self.advance_and_emit(start, 2, TokenKind::BoolOr),
                Some(b'=') => self.advance_and_emit(start, 2, TokenKind::BitOrAssign),
                _ => self.advance_and_emit(start, 1, TokenKind::BitOr),
            },
            Some(b'^') => match self.peek(1) {
                Some(b'=') => self.advance_and_emit(start, 2, TokenKind::BitXorAssign),
                _ => self.advance_and_emit(start, 1, TokenKind::BitXor),
            },
            Some(b'!') => match self.peek(1) {
                Some(b'=') => self.advance_and_emit(start, 2, TokenKind::Neq),
                _ => self.advance_and_emit(start, 1, TokenKind::BoolNot),
            },
            Some(b'<') => match self.peek(1) {
                Some(b'<') => match self.peek(2) {
                    Some(b'=') => self.advance_and_emit(start, 3, TokenKind::BitShlAssign),
                    _ => self.advance_and_emit(start, 2, TokenKind::BitShl),
                },
                Some(b'=') => self.advance_and_emit(start, 2, TokenKind::LtEq),
                _ => self.advance_and_emit(start, 1, TokenKind::Lt),
            },
            Some(b'>') => match self.peek(1) {
                Some(b'>') => match self.peek(2) {
                    Some(b'=') => self.advance_and_emit(start, 3, TokenKind::BitShrAssign),
                    _ => self.advance_and_emit(start, 2, TokenKind::BitShr),
                },
                Some(b'=') => self.advance_and_emit(start, 2, TokenKind::GtEq),
                _ => self.advance_and_emit(start, 1, TokenKind::Gt),
            },
            Some(b'=') => match self.peek(1) {
                Some(b'=') => self.advance_and_emit(start, 2, TokenKind::Eq),
                _ => self.advance_and_emit(start, 1, TokenKind::Assign),
            },
            Some(b'(') => self.advance_and_emit(start, 1, TokenKind::LeftParen),
            Some(b')') => self.advance_and_emit(start, 1, TokenKind::RightParen),
            Some(b'[') => self.advance_and_emit(start, 1, TokenKind::LeftBracket),
            Some(b']') => self.advance_and_emit(start, 1, TokenKind::RightBracket),
            Some(b'{') => self.advance_and_emit(start, 1, TokenKind::LeftBrace),
            Some(b'}') => self.advance_and_emit(start, 1, TokenKind::RightBrace),
            Some(b';') => self.advance_and_emit(start, 1, TokenKind::Semi),
            _ => self.scan_unknown_token(),
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(0), Some(b' ' | b'\r' | b'\n' | b'\t')) {
            self.advance();
        }
    }

    fn scan_eof(&self) -> LexerTokenResult<'src> {
        self.emit(self.offset, TokenKind::Eof)
    }

    fn scan_ident(&mut self) -> LexerTokenResult<'src> {
        let start = self.offset;
        while let Some(byte) = self.peek(0)
            && (byte.is_ascii_alphanumeric() || byte == b'_')
        {
            self.advance()
        }

        let text = self.slice_to_str(start, self.offset)?;
        let kind = match text {
            "fn" => TokenKind::Fn,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "for" => TokenKind::For,
            "while" => TokenKind::While,
            "return" => TokenKind::Return,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "let" => TokenKind::Let,
            "true" => TokenKind::BoolLit(true),
            "false" => TokenKind::BoolLit(false),
            _ => TokenKind::Ident,
        };
        self.emit(start, kind)
    }

    fn scan_num_lit(&mut self) -> LexerTokenResult<'src> {
        match self.peek(0) {
            Some(b'0') => match self.peek(1) {
                Some(b'b' | b'B') => self.scan_bin_int_lit(),
                Some(b'o' | b'O') => self.scan_oct_int_lit(),
                Some(b'x' | b'X') => self.scan_hex_int_lit(),
                _ => self.scan_dec_num_lit(),
            },
            _ => self.scan_dec_num_lit(),
        }
    }

    fn scan_bin_int_lit(&mut self) -> LexerTokenResult<'src> {
        let start = self.offset;

        // Advance over the 0b prefix.
        self.advance_n(2);

        while matches!(self.peek(0), Some(b'0' | b'1')) {
            self.advance();
        }

        let text = self.slice_to_str(start, self.offset)?;
        self.parse_int_lit(start, 2, text, 2)
    }

    fn scan_oct_int_lit(&mut self) -> LexerTokenResult<'src> {
        let start = self.offset;

        // Advance over the 0o prefix.
        self.advance_n(2);

        while matches!(self.peek(0), Some(b'0'..=b'7')) {
            self.advance();
        }

        let text = self.slice_to_str(start, self.offset)?;
        self.parse_int_lit(start, 2, text, 8)
    }

    fn scan_dec_num_lit(&mut self) -> LexerTokenResult<'src> {
        enum DecState {
            Int,
            Frac,
            ExpSign,
            ExpValue,
        }

        let start = self.offset;
        let mut state = DecState::Int;

        loop {
            match state {
                DecState::Int => match self.peek(0) {
                    Some(b'0'..=b'9') => {}
                    Some(b'e' | b'E') => state = DecState::ExpSign,
                    Some(b'.') => state = DecState::Frac,
                    _ => break,
                },
                DecState::Frac => match self.peek(0) {
                    Some(b'0'..=b'9') => {}
                    Some(b'e' | b'E') => state = DecState::ExpSign,
                    _ => break,
                },
                DecState::ExpSign => match self.peek(0) {
                    Some(b'+' | b'-' | b'0'..=b'9') => state = DecState::ExpValue,
                    _ => break,
                },
                DecState::ExpValue => match self.peek(0) {
                    Some(b'0'..=b'9') => {}
                    _ => break,
                },
            }
            self.advance();
        }

        let text = self.slice_to_str(start, self.offset)?;

        if matches!(state, DecState::Int) {
            self.parse_int_lit(start, 0, text, 10)
        } else {
            self.parse_float_lit(start, text)
        }
    }

    fn scan_hex_int_lit(&mut self) -> LexerTokenResult<'src> {
        let start = self.offset;

        // Advance over the 0x prefix.
        self.advance_n(2);

        while matches!(self.peek(0), Some(b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')) {
            self.advance();
        }

        let text = self.slice_to_str(start, self.offset)?;
        self.parse_int_lit(start, 2, text, 16)
    }

    fn parse_int_lit(
        &self,
        start: usize,
        prefix_offset: usize,
        text: &'src str,
        radix: u32,
    ) -> LexerTokenResult<'src> {
        match u64::from_str_radix(&text[prefix_offset..], radix) {
            Ok(value) => self.emit(start, TokenKind::IntLit(value)),
            Err(_) => self.emit_error(start, LexerError::InvalidIntLiteral),
        }
    }

    fn parse_float_lit(&self, start: usize, text: &'src str) -> LexerTokenResult<'src> {
        match f64::from_str(text) {
            Ok(value) if value.is_finite() => self.emit(start, TokenKind::FloatLit(value)),
            Ok(_) => self.emit_error(start, LexerError::FloatLiteralIsInfinite),
            Err(_) => self.emit_error(start, LexerError::InvalidFloatLiteral),
        }
    }

    fn scan_str_lit(&mut self) -> LexerTokenResult<'src> {
        let start = self.offset;

        // Advance over the opening quote.
        self.advance();

        // We only ensure that strings are closed in the lexer. This includes ignoring \" sequences.
        // Everything else will be managed in the parser later on (which allows us to do things
        // like string interpolation in the future).
        loop {
            match self.peek(0) {
                Some(b'\r' | b'\n') | None => {
                    return self.emit_error(start, LexerError::UnclosedStringLiteral);
                }
                // Step over any occurrence of '\"' within a string.
                Some(b'\\') if matches!(self.peek(1), Some(b'"')) => self.advance_n(2),
                // Advance over the closing quote and stop iterating.
                Some(b'"') => {
                    self.advance();
                    break;
                }
                _ => self.advance(),
            }
        }

        self.emit(start, TokenKind::StrLit)
    }

    fn scan_single_line_comment(&mut self) -> LexerTokenResult<'src> {
        let start = self.offset;

        // Step over the '//'
        self.advance_n(2);

        while !matches!(self.peek(0), None | Some(b'\r' | b'\n')) {
            self.advance();
        }

        self.emit(start, TokenKind::SingleLineComment)
    }

    fn scan_multi_line_comment(&mut self) -> LexerTokenResult<'src> {
        let start = self.offset;

        // Step over the '/*'
        self.advance_n(2);

        loop {
            match self.peek(0) {
                Some(b'*') if matches!(self.peek(1), Some(b'/')) => {
                    // Step over the '*/'
                    self.advance_n(2);
                    break;
                }
                Some(_) => self.advance(),
                None => return self.emit_error(start, LexerError::UnclosedMultiLineComment),
            }
        }

        self.emit(start, TokenKind::MultiLineComment)
    }

    fn scan_unknown_token(&mut self) -> LexerTokenResult<'src> {
        let start = self.offset;
        self.advance();

        while let Some(byte) = self.peek(0) {
            // If the MSB is 1, it is probably a Unicode continuation character, so consume it as
            // well. Consume alphanumeric symbols and control sequences that are not whitespace.
            // This allows the lexer to recover such that the next token is something we can most
            // likely understand again.
            match byte {
                b if b.is_ascii_control() && !b.is_ascii_whitespace() => self.advance(),
                b if b.is_ascii_alphanumeric() => self.advance(),
                b if b > 0x7F => self.advance(),
                _ => break,
            }
        }

        let invalid_text = self.slice_to_str(start, self.offset)?;

        self.emit_error(start, LexerError::UnknownToken(invalid_text))
    }

    #[inline(always)]
    fn peek(&self, offset: usize) -> Option<u8> {
        self.src.get(self.offset + offset).copied()
    }

    fn advance(&mut self) {
        if self.offset < self.src.len() {
            let byte = self.src[self.offset];

            self.offset += 1;

            if byte == b'\n' {
                self.line_starts.push(self.offset);
            }
        }
    }

    #[inline(always)]
    fn advance_n(&mut self, n: usize) {
        for _ in 0..n {
            self.advance();
        }
    }

    fn slice_to_str(&self, start: usize, end: usize) -> LexerResult<'src, &'src str> {
        let raw = &self.src[start..end];
        match str::from_utf8(raw) {
            Ok(value) => Ok(value),
            Err(_) => self.emit_error(start, LexerError::InvalidUnicodeSequence(raw)),
        }
    }

    fn emit(&self, start: usize, kind: TokenKind) -> LexerTokenResult<'src> {
        let raw_content = self.slice_to_str(start, self.offset)?;

        Ok(Spanned::new(
            Token::new(raw_content, kind),
            Span::new(start, self.offset),
        ))
    }

    #[inline(always)]
    fn advance_and_emit(
        &mut self,
        start: usize,
        len: usize,
        kind: TokenKind,
    ) -> LexerTokenResult<'src> {
        self.advance_n(len);
        self.emit(start, kind)
    }

    fn emit_error<R>(&self, start: usize, error: LexerError<'src>) -> LexerResult<'src, R> {
        Err(Spanned::new(error, Span::new(start, self.offset)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(                                          "",                     TokenKind::Eof ; "end of file")]
    #[test_case(          "// This is a single line comment",       TokenKind::SingleLineComment ; "single line comment")]
    #[test_case("/* This is a \n\n mutli-line \n comment */",        TokenKind::MultiLineComment ; "multi-line comment")]
    #[test_case(                                      "true",           TokenKind::BoolLit(true) ; "true boolean literal")]
    #[test_case(                                     "false",          TokenKind::BoolLit(false) ; "false boolean literal")]
    #[test_case(                         "\"hello, world!\"",                  TokenKind::StrLit ; "string literal")]
    #[test_case(  "\"i said \\\"hello, world!\\\" to them\"",                  TokenKind::StrLit ; "string literal with escape sequences")]
    #[test_case(                                    "0b1101",          TokenKind::IntLit(0b1101) ; "base 2 int literal (lowercase prefix)")]
    #[test_case(                           "0B0010110100101", TokenKind::IntLit(0b0010110100101) ; "base 2 int literal (capital prefix)")]
    #[test_case(                                 "0o1234567",       TokenKind::IntLit(0o1234567) ; "base 8 int literal (lowercase prefix)")]
    #[test_case(                                "0O76543210",      TokenKind::IntLit(0o76543210) ; "base 8 int literal (capital prefix)")]
    #[test_case(                                        "69",              TokenKind::IntLit(69) ; "base 10 int literal")]
    #[test_case(                            "0x1a2b3c4d5e6f",  TokenKind::IntLit(0x1a2b3c4d5e6f) ; "base 16 int literal (lowercase prefix)")]
    #[test_case(                            "0XDEADBEEF6942",  TokenKind::IntLit(0xdeadbeef6942) ; "base 16 int literal (uppercase prefix)")]
    #[test_case(                                     "0.123",         TokenKind::FloatLit(0.123) ; "float literal with integer and fraction")]
    #[test_case(                                        "5.",           TokenKind::FloatLit(5.0) ; "float literal with integer, and empty fraction")]
    #[test_case(                                      ".553",         TokenKind::FloatLit(0.553) ; "float literal with empty integer, and fraction")]
    #[test_case(                                     "12e15",         TokenKind::FloatLit(12e15) ; "float literal with integer, and unsigned exponent (lowercase e)")]
    #[test_case(                                     "13E15",         TokenKind::FloatLit(13e15) ; "float literal with integer, and unsigned exponent (uppercase e)")]
    #[test_case(                                    ".553e3",       TokenKind::FloatLit(0.553e3) ; "float literal with empty integer, fraction, and unsigned exponent (lowercase e)")]
    #[test_case(                                    ".553E3",       TokenKind::FloatLit(0.553e3) ; "float literal with empty integer, fraction, and unsigned exponent (uppercase e)")]
    #[test_case(                                     "2e+14",          TokenKind::FloatLit(2e14) ; "float literal with integer, and positive signed exponent (lowercase e")]
    #[test_case(                                     "3E+16",          TokenKind::FloatLit(3e16) ; "float literal with integer, and positive signed exponent (uppercase e")]
    #[test_case(                                   ".553e+3",       TokenKind::FloatLit(0.553e3) ; "float literal with empty integer, fraction, and positive signed exponent (lowercase e)")]
    #[test_case(                                   ".553E+3",       TokenKind::FloatLit(0.553e3) ; "float literal with empty integer, fraction, and positive signed exponent (uppercase e)")]
    #[test_case(                                     "2e-14",         TokenKind::FloatLit(2e-14) ; "float literal with integer, and negative signed exponent (lowercase e")]
    #[test_case(                                     "3E-16",         TokenKind::FloatLit(3e-16) ; "float literal with integer, and negative signed exponent (uppercase e")]
    #[test_case(                                   ".553e-3",      TokenKind::FloatLit(0.553e-3) ; "float literal with empty integer, fraction, and negative signed exponent (lowercase e)")]
    #[test_case(                                   ".553E-3",      TokenKind::FloatLit(0.553e-3) ; "float literal with empty integer, fraction, and negative signed exponent (uppercase e)")]
    #[test_case(                                 "12.718e15",     TokenKind::FloatLit(12.718e15) ; "float literal with integer, fraction, and unsigned exponent (lowercase e)")]
    #[test_case(                                 "13.718E15",     TokenKind::FloatLit(13.718e15) ; "float literal with integer, fraction, and unsigned exponent (uppercase e)")]
    #[test_case(                                 "2.718e+14",      TokenKind::FloatLit(2.718e14) ; "float literal with integer, fraction, and positive signed exponent (lowercase e)")]
    #[test_case(                                 "3.718E+16",      TokenKind::FloatLit(3.718e16) ; "float literal with integer, fraction, and positive signed exponent (uppercase e)")]
    #[test_case(                                 "2.718e-14",     TokenKind::FloatLit(2.718e-14) ; "float literal with integer, fraction, and negative signed exponent (lowercase e)")]
    #[test_case(                                 "3.718E-16",     TokenKind::FloatLit(3.718e-16) ; "float literal with integer, fraction, and negative signed exponent (uppercase e)")]
    #[test_case(                                         "_",                   TokenKind::Ident ; "unnamed identifier")]
    #[test_case(                                 "lowercase",                   TokenKind::Ident ; "lowercase identifier")]
    #[test_case(                                 "UPPERCASE",                   TokenKind::Ident ; "uppercase identifier")]
    #[test_case(                                      "i123",                   TokenKind::Ident ; "lowercase identifier with numbers")]
    #[test_case(                                      "I123",                   TokenKind::Ident ; "uppercase identifier with numbers")]
    #[test_case(                            "snake_case_123",                   TokenKind::Ident ; "snake case identifier")]
    #[test_case(                  "SCREAMING_SNAKE_CASE_123",                   TokenKind::Ident ; "screaming snake case identifier")]
    #[test_case(                              "camelCase123",                   TokenKind::Ident ; "camel case identifier")]
    #[test_case(                             "PascalCase123",                   TokenKind::Ident ; "pascal case identifier")]
    #[test_case(                                        "fn",                      TokenKind::Fn ; "fn keyword")]
    #[test_case(                                        "if",                      TokenKind::If ; "if keyword")]
    #[test_case(                                      "else",                    TokenKind::Else ; "else keyword")]
    #[test_case(                                     "while",                   TokenKind::While ; "while keyword")]
    #[test_case(                                       "for",                     TokenKind::For ; "for keyword")]
    #[test_case(                                    "return",                  TokenKind::Return ; "return keyword")]
    #[test_case(                                     "break",                   TokenKind::Break ; "break keyword")]
    #[test_case(                                  "continue",                TokenKind::Continue ; "continue keyword")]
    #[test_case(                                       "let",                     TokenKind::Let ; "let keyword")]
    #[test_case(                                         "+",                     TokenKind::Add ; "addition operator")]
    #[test_case(                                         "-",                     TokenKind::Sub ; "subtraction operator")]
    #[test_case(                                         "*",                     TokenKind::Mul ; "multiplication operator")]
    #[test_case(                                         "/",                     TokenKind::Div ; "division operator")]
    #[test_case(                                         "%",                     TokenKind::Mod ; "modulo operator")]
    #[test_case(                                        "**",                     TokenKind::Pow ; "power operator")]
    #[test_case(                                        "+=",               TokenKind::AddAssign ; "assignment addition operator")]
    #[test_case(                                        "-=",               TokenKind::SubAssign ; "assignment subtraction operator")]
    #[test_case(                                        "*=",               TokenKind::MulAssign ; "assignment multiplication operator")]
    #[test_case(                                        "/=",               TokenKind::DivAssign ; "assignment division operator")]
    #[test_case(                                        "%=",               TokenKind::ModAssign ; "assignment modulo operator")]
    #[test_case(                                       "**=",               TokenKind::PowAssign ; "assignment power operator")]
    #[test_case(                                         "~",                  TokenKind::BitNot ; "binary inversion operator")]
    #[test_case(                                         "&",                  TokenKind::BitAnd ; "binary and operator")]
    #[test_case(                                         "|",                   TokenKind::BitOr ; "binary or operator")]
    #[test_case(                                         "^",                  TokenKind::BitXor ; "binary xor operator")]
    #[test_case(                                        "<<",                  TokenKind::BitShl ; "binary left-bitshift operator")]
    #[test_case(                                        ">>",                  TokenKind::BitShr ; "binary right-bitshift operator")]
    #[test_case(                                        "&=",            TokenKind::BitAndAssign ; "assignment binary and operator")]
    #[test_case(                                        "|=",             TokenKind::BitOrAssign ; "assignment binary or operator")]
    #[test_case(                                        "^=",            TokenKind::BitXorAssign ; "assignment binary xor operator")]
    #[test_case(                                       "<<=",            TokenKind::BitShlAssign ; "assignment binary left-bitshift operator")]
    #[test_case(                                       ">>=",            TokenKind::BitShrAssign ; "assignment binary right-bitshift operator")]
    #[test_case(                                         "!",                 TokenKind::BoolNot ; "boolean not operator")]
    #[test_case(                                        "&&",                 TokenKind::BoolAnd ; "boolean and operator")]
    #[test_case(                                        "||",                  TokenKind::BoolOr ; "boolean or operator")]
    #[test_case(                                        "==",                      TokenKind::Eq ; "equality operator")]
    #[test_case(                                        "!=",                     TokenKind::Neq ; "inequality operator")]
    #[test_case(                                         "<",                      TokenKind::Lt ; "less-than operator")]
    #[test_case(                                        "<=",                    TokenKind::LtEq ; "less-than-or-equal operator")]
    #[test_case(                                         ">",                      TokenKind::Gt ; "greater-than operator")]
    #[test_case(                                        ">=",                    TokenKind::GtEq ; "greater-than-or-equal operator")]
    #[test_case(                                         "=",                  TokenKind::Assign ; "assignment operator")]
    #[test_case(                                         "(",               TokenKind::LeftParen ; "left parenthesis")]
    #[test_case(                                         ")",              TokenKind::RightParen ; "right parenthesis")]
    #[test_case(                                         "[",             TokenKind::LeftBracket ; "left bracket")]
    #[test_case(                                         "]",            TokenKind::RightBracket ; "right bracket")]
    #[test_case(                                         "{",               TokenKind::LeftBrace ; "left brace")]
    #[test_case(                                         "}",              TokenKind::RightBrace ; "right brace")]
    #[test_case(                                         ";",                    TokenKind::Semi ; "semicolon")]
    fn tokens_are_scanned_as_expected(input: &str, expected_kind: TokenKind) {
        // Given
        let raw_input = input.as_bytes();
        let mut lexer = Lexer::new(raw_input);

        // When
        let spanned_token = lexer.next_token().expect("unexpected lexer error");

        // Then
        assert_eq!(spanned_token.span(), Span::new(0, raw_input.len()), "span");
        assert_eq!(
            spanned_token.value().raw_content(),
            input,
            "token.raw_content"
        );
        assert_eq!(spanned_token.value().kind(), expected_kind, "token.kind");
    }

    // Verify "fusion" - i.e. we repeatedly yield EOF correctly once at the end of the file rather
    // than going out of bounds anywhere.
    #[test]
    fn eof_tokens_fuse() {
        // Given
        let raw_input = "".as_bytes();
        let mut lexer = Lexer::new(raw_input);

        // Then
        for _ in 1..10 {
            let spanned_token = lexer.next_token().expect("unexpected lexer error");
            assert_eq!(spanned_token.span(), Span::new(0, raw_input.len()), "span");
            assert_eq!(spanned_token.value().raw_content(), "", "token.raw_content");
            assert_eq!(spanned_token.value().kind(), TokenKind::Eof, "token.kind");
        }
    }

    #[test_case(                                                                 "0b",  2 ; "binary prefix without value (lowercase)")]
    #[test_case(                                                                 "0o",  2 ; "octal prefix without value (lowercase)")]
    #[test_case(                                                                 "0x",  2 ; "hexadecimal prefix without value (lowercase)")]
    #[test_case(                                                                 "0B",  2 ; "binary prefix without value (uppercase)")]
    #[test_case(                                                                 "0O",  2 ; "octal prefix without value (uppercase)")]
    #[test_case(                                                                 "0X",  2 ; "hexadecimal prefix without value (uppercase)")]
    #[test_case(                                                                "0b9",  2 ; "binary prefix, invalid base value")]
    #[test_case(                                                                "0o8",  2 ; "octal prefix, invalid base value")]
    #[test_case(                                                                "0xg",  2 ; "hexadecimal prefix, invalid base value")]
    #[test_case("0b11111111111111111111111111111111111111111111111111111111111111110", 67 ; "binary value, too large")]
    #[test_case(                                          "0o20000000000000000000000", 25 ; "octal value, too large")]
    #[test_case(                                               "18446744073709551616", 20 ; "decimal value, too large")]
    #[test_case(                                                "0x10000000000000000", 19 ; "hexadecimal value, too large")]
    fn lexer_errors_on_invalid_ints(input: &str, expected_length: usize) {
        // Given
        let raw_input = input.as_bytes();
        let mut lexer = Lexer::new(raw_input);

        // When
        let error = lexer
            .next_token()
            .expect_err("expected error parsing token");

        // Then
        assert_eq!(error.span(), Span::new(0, expected_length), "error.span");
        assert!(
            matches!(error.value(), LexerError::InvalidIntLiteral),
            "error.value {:?}",
            error.value()
        );
    }

    // We do not test for too-small values currently. Rust will implicitly translate them to 0 and
    // we cannot catch this easily right now.
    #[test_case(  "1.0e",    LexerError::InvalidFloatLiteral ; "missing unsigned exponent value")]
    #[test_case( "1.0e+",    LexerError::InvalidFloatLiteral ; "missing positive signed exponent value")]
    #[test_case( "1.0e-",    LexerError::InvalidFloatLiteral ; "missing negative signed exponent value")]
    #[test_case("5e+309", LexerError::FloatLiteralIsInfinite ; "too big to fit in f64")]
    fn lexer_errors_on_invalid_floats(input: &str, expected_error: LexerError) {
        // Given
        let raw_input = input.as_bytes();
        let mut lexer = Lexer::new(raw_input);

        // When
        let error = lexer
            .next_token()
            .expect_err("expected error parsing token");

        // Then
        assert_eq!(error.span(), Span::new(0, input.len()), "error.span");
        assert_eq!(
            error.value(),
            &expected_error,
            "error.value {:?}",
            error.value()
        );
    }

    #[test]
    fn lexer_errors_on_unclosed_multi_line_comments_at_eof() {
        // Given
        let raw_input = b"/* foo";
        let mut lexer = Lexer::new(raw_input);

        // When
        let error = lexer
            .next_token()
            .expect_err("expected error parsing token");

        // Then
        assert_eq!(error.span(), Span::new(0, raw_input.len()), "error.span");
        assert!(
            matches!(error.value(), LexerError::UnclosedMultiLineComment),
            "error.value {:?}",
            error.value()
        );
    }

    #[test]
    fn lexer_errors_on_unclosed_strings_at_eof() {
        // Given
        let raw_input = b"\"string";
        let mut lexer = Lexer::new(raw_input);

        // When
        let error = lexer
            .next_token()
            .expect_err("expected error parsing token");

        // Then
        assert_eq!(error.span(), Span::new(0, raw_input.len()), "error.span");
        assert!(
            matches!(error.value(), LexerError::UnclosedStringLiteral),
            "error.value {:?}",
            error.value()
        );
    }

    #[test]
    fn lexer_errors_on_unclosed_strings_at_end_of_line() {
        // Given
        let raw_input = b"\"string\nfoo";
        let mut lexer = Lexer::new(raw_input);

        // When
        let error = lexer
            .next_token()
            .expect_err("expected error parsing token");
        let token1 = lexer
            .next_token()
            .expect("expected success parsing token after erroneous string");

        // Then
        assert_eq!(error.span(), Span::new(0, 7), "error.span");
        assert!(
            matches!(error.value(), LexerError::UnclosedStringLiteral),
            "error.value {:?}",
            error.value()
        );

        assert_eq!(token1.span(), Span::new(8, 11), "token1.span");
        assert_eq!(token1.value().kind(), TokenKind::Ident, "token1.value.kind");
        assert_eq!(
            token1.value().raw_content(),
            "foo",
            "token1.value.raw_content"
        );
    }

    #[test]
    fn lexer_errors_on_invalid_tokens_and_recovers() {
        // Given
        let raw_input = "aa $foo bb".as_bytes();
        let mut lexer = Lexer::new(raw_input);

        // When
        let token1 = lexer
            .next_token()
            .expect("expected success parsing 'aa' token");
        let error = lexer
            .next_token()
            .expect_err("expected an error parsing '$foo' token");
        let token2 = lexer
            .next_token()
            .expect("expected success parsing 'bb' token");

        // Then
        assert_eq!(token1.span(), Span::new(0, 2), "token1.span");
        assert_eq!(
            token1.value().raw_content(),
            "aa",
            "token1.value.raw_content"
        );
        assert_eq!(error.span(), Span::new(3, 7), "error.span");
        assert_eq!(
            error.value(),
            &LexerError::UnknownToken("$foo"),
            "error.value"
        );
        assert_eq!(token2.span(), Span::new(8, 10), "token2.span");
        assert_eq!(
            token2.value().raw_content(),
            "bb",
            "token2.value.raw_content"
        );
    }

    #[test]
    fn lexer_errors_on_invalid_unicode_codepoints_and_recovers() {
        // Given
        let raw_input = b"aa foo\xff\x0fbar bb";
        let mut lexer = Lexer::new(raw_input);

        // When
        let token1 = lexer
            .next_token()
            .expect("expected success parsing 'aa' token");
        let token2 = lexer
            .next_token()
            .expect("expected success parsing 'foo' token");
        let error = lexer
            .next_token()
            .expect_err("expected an error parsing invalid unicode token");
        let token3 = lexer
            .next_token()
            .expect("expected success parsing 'bb' token");

        // Then
        assert_eq!(token1.span(), Span::new(0, 2), "token1.span");
        assert_eq!(
            token1.value().raw_content(),
            "aa",
            "token1.value.raw_content"
        );
        assert_eq!(token2.span(), Span::new(3, 6), "token2.span");
        assert_eq!(
            token2.value().raw_content(),
            "foo",
            "token2.value.raw_content"
        );
        assert_eq!(error.span(), Span::new(6, 11), "error.span");
        assert_eq!(
            error.value(),
            &LexerError::InvalidUnicodeSequence(b"\xff\x0fbar"),
            "error.value"
        );
        assert_eq!(token3.span(), Span::new(12, 14), "token3.span");
        assert_eq!(
            token3.value().raw_content(),
            "bb",
            "token3.value.raw_content"
        );
    }

    #[test]
    fn lexer_advances_as_expected() {
        // Given
        let raw_input = "foo+bar/baz".as_bytes();
        let mut lexer = Lexer::new(raw_input);

        // When
        let foo = lexer.next_token().expect("error parsing foo token");
        let add = lexer.next_token().expect("error parsing add token");
        let bar = lexer.next_token().expect("error parsing bar token");
        let div = lexer.next_token().expect("error parsing divide token");
        let baz = lexer.next_token().expect("error parsing baz token");
        let eof = lexer.next_token().expect("error parsing eof token");

        // Then
        assert_eq!(foo.span(), Span::new(0, 3), "foo.span");
        assert_eq!(foo.value().kind(), TokenKind::Ident, "foo.value.kind");
        assert_eq!(foo.value().raw_content(), "foo", "foo.value.raw_content");

        assert_eq!(add.span(), Span::new(3, 4), "add.span");
        assert_eq!(add.value().kind(), TokenKind::Add, "add.value.kind");
        assert_eq!(add.value().raw_content(), "+", "add.value.raw_content");

        assert_eq!(bar.span(), Span::new(4, 7), "bar.span");
        assert_eq!(bar.value().kind(), TokenKind::Ident, "bar.value.kind");
        assert_eq!(bar.value().raw_content(), "bar", "bar.value.raw_content");

        assert_eq!(div.span(), Span::new(7, 8), "div.span");
        assert_eq!(div.value().kind(), TokenKind::Div, "div.value.kind");
        assert_eq!(div.value().raw_content(), "/", "div.value.raw_content");

        assert_eq!(baz.span(), Span::new(8, 11), "baz.span");
        assert_eq!(baz.value().kind(), TokenKind::Ident, "baz.value.kind");
        assert_eq!(baz.value().raw_content(), "baz", "baz.value.raw_content");

        assert_eq!(eof.span(), Span::new(11, 11), "eof.span");
        assert_eq!(eof.value().kind(), TokenKind::Eof, "eof.value.kind");
        assert_eq!(eof.value().raw_content(), "", "eof.value.raw_content");
    }

    #[test]
    fn lexer_skips_whitespaces_as_expected() {
        // Given
        let raw_input = "foo + \t\t\r\n\r bar \t\t /    baz  ".as_bytes();
        let mut lexer = Lexer::new(raw_input);

        // When
        let foo = lexer.next_token().expect("error parsing foo token");
        let add = lexer.next_token().expect("error parsing add token");
        let bar = lexer.next_token().expect("error parsing bar token");
        let div = lexer.next_token().expect("error parsing divide token");
        let baz = lexer.next_token().expect("error parsing baz token");
        let eof = lexer.next_token().expect("error parsing eof token");

        // Then
        assert_eq!(foo.span(), Span::new(0, 3), "foo.span");
        assert_eq!(foo.value().kind(), TokenKind::Ident, "foo.value.kind");
        assert_eq!(foo.value().raw_content(), "foo", "foo.value.raw_content");

        assert_eq!(add.span(), Span::new(4, 5), "add.span");
        assert_eq!(add.value().kind(), TokenKind::Add, "add.value.kind");
        assert_eq!(add.value().raw_content(), "+", "add.value.raw_content");

        assert_eq!(bar.span(), Span::new(12, 15), "bar.span");
        assert_eq!(bar.value().kind(), TokenKind::Ident, "bar.value.kind");
        assert_eq!(bar.value().raw_content(), "bar", "bar.value.raw_content");

        assert_eq!(div.span(), Span::new(19, 20), "div.span");
        assert_eq!(div.value().kind(), TokenKind::Div, "div.value.kind");
        assert_eq!(div.value().raw_content(), "/", "div.value.raw_content");

        assert_eq!(baz.span(), Span::new(24, 27), "baz.span");
        assert_eq!(baz.value().kind(), TokenKind::Ident, "baz.value.kind");
        assert_eq!(baz.value().raw_content(), "baz", "baz.value.raw_content");

        assert_eq!(eof.span(), Span::new(29, 29), "eof.span");
        assert_eq!(eof.value().kind(), TokenKind::Eof, "eof.value.kind");
        assert_eq!(eof.value().raw_content(), "", "eof.value.raw_content");
    }

    #[test]
    fn lexer_tracks_newlines_as_expected() {
        // Given
        let raw_input = "foo\n\n//comment\r\nbar\nbaz bork\nqux\n  quxx".as_bytes();
        let mut lexer = Lexer::new(raw_input);

        // When
        let foo = lexer.next_token().expect("error parsing foo token");
        let comment = lexer.next_token().expect("error parsing comment token");
        let bar = lexer.next_token().expect("error parsing bar token");
        let baz = lexer.next_token().expect("error parsing baz token");
        let bork = lexer.next_token().expect("error parsing bork token");
        let qux = lexer.next_token().expect("error parsing qux token");
        let quxx = lexer.next_token().expect("error parsing quxx token");
        let eof = lexer.next_token().expect("error parsing eof token");

        // Then
        assert_eq!(
            lexer.get_line_number_of(foo.span()),
            (0, 0),
            "foo line number and line offset"
        );
        assert_eq!(
            lexer.get_line_number_of(comment.span()),
            (2, 5),
            "comment line number and line offset"
        );
        assert_eq!(
            lexer.get_line_number_of(bar.span()),
            (3, 16),
            "bar line number and line offset"
        );
        assert_eq!(
            lexer.get_line_number_of(baz.span()),
            (4, 20),
            "baz line number and line offset"
        );
        assert_eq!(
            lexer.get_line_number_of(bork.span()),
            (4, 20),
            "bork line number and line offset"
        );
        assert_eq!(
            lexer.get_line_number_of(qux.span()),
            (5, 29),
            "qux line number and line offset"
        );
        assert_eq!(
            lexer.get_line_number_of(quxx.span()),
            (6, 33),
            "quxx line number and line offset"
        );
        assert_eq!(
            lexer.get_line_number_of(eof.span()),
            (6, 33),
            "eof line number and line offset"
        );
    }
}
