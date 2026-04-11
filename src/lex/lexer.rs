use crate::lex::error::LexerError;
use crate::lex::token::{Token, TokenKind};
use crate::span::{Span, Spanned};
use core::str;
use std::str::FromStr;

#[derive(Debug)]
pub struct Lexer<'src> {
    src: &'src [u8],
    offset: usize,

    // Store the offsets for the start of each line as we hit it. The parser can query this
    // later to find the exact line number when producing an error message, given the offset
    // in a span.
    line_starts: Vec<usize>,
}

type LexerResult<'src> = Result<Spanned<Token<'src>>, Spanned<LexerError<'src>>>;

impl<'src> Lexer<'src> {
    pub fn new(src: &'src [u8]) -> Self {
        Self {
            src,
            offset: 0,
            line_starts: vec![0],
        }
    }

    pub fn get_line_number_of(&self, span: Span) -> usize {
        self.line_starts
            .partition_point(|&start| start <= span.start)
    }

    pub fn next_token(&mut self) -> LexerResult<'src> {
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
                Some(b'*') => self.scan_multiline_comment(),
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
            _ => self.scan_invalid_character(),
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(0), Some(b' ' | b'\r' | b'\n' | b'\t')) {
            self.advance();
        }
    }

    fn scan_eof(&self) -> LexerResult<'src> {
        self.emit(self.offset, TokenKind::Eof)
    }

    fn scan_ident(&mut self) -> LexerResult<'src> {
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

    fn scan_num_lit(&mut self) -> LexerResult<'src> {
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

    fn scan_bin_int_lit(&mut self) -> LexerResult<'src> {
        let start = self.offset;

        // Advance over the 0b prefix.
        self.advance_n(2);

        while matches!(self.peek(0), Some(b'0' | b'1')) {
            self.advance();
        }

        let text = self.slice_to_str(start, self.offset)?;
        self.parse_int_lit(start, 2, text, 2)
    }

    fn scan_oct_int_lit(&mut self) -> LexerResult<'src> {
        let start = self.offset;

        // Advance over the 0o prefix.
        self.advance_n(2);

        while matches!(self.peek(0), Some(b'0'..=b'7')) {
            self.advance();
        }

        let text = self.slice_to_str(start, self.offset)?;
        self.parse_int_lit(start, 2, text, 8)
    }

    fn scan_dec_num_lit(&mut self) -> LexerResult<'src> {
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

    fn scan_hex_int_lit(&mut self) -> LexerResult<'src> {
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
    ) -> LexerResult<'src> {
        match u64::from_str_radix(&text[prefix_offset..], radix) {
            Ok(value) => self.emit(start, TokenKind::IntLit(value)),
            Err(error) => self.emit_error(start, LexerError::IntParseError(error)),
        }
    }

    fn parse_float_lit(&self, start: usize, text: &'src str) -> LexerResult<'src> {
        match f64::from_str(text) {
            Ok(value) => self.emit(start, TokenKind::FloatLit(value)),
            Err(error) => self.emit_error(start, LexerError::FloatParseError(error)),
        }
    }

    fn scan_str_lit(&mut self) -> LexerResult<'src> {
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

    fn scan_single_line_comment(&mut self) -> LexerResult<'src> {
        let start = self.offset;

        // Step over the '//'
        self.advance_n(2);

        while !matches!(self.peek(0), None | Some(b'\r' | b'\n')) {
            self.advance();
        }

        self.emit(start, TokenKind::SingleLineComment)
    }

    fn scan_multiline_comment(&mut self) -> LexerResult<'src> {
        let start = self.offset;

        // Step over the '/*'
        self.advance_n(2);

        loop {
            match self.peek(0) {
                // Be pedantic and complain if we forget to close a multiline comment before EOF.
                None => return self.emit_error(start, LexerError::UnclosedMultilineComment),
                Some(b'*') if matches!(self.peek(1), Some(b'/')) => {
                    // Step over the '*/'
                    self.advance_n(2);
                    break;
                }
                _ => self.advance(),
            }
        }

        self.emit(start, TokenKind::MultiLineComment)
    }

    fn scan_invalid_character(&mut self) -> LexerResult<'src> {
        let start = self.offset;
        self.advance();

        // If the first two bits are '10', it is a Unicode continuation character, so consume
        // it as well.
        while let Some(byte) = self.peek(0)
            && (byte & 0xC0 == 0x80)
        {
            self.advance();
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

    fn slice_to_str(
        &self,
        start: usize,
        end: usize,
    ) -> Result<&'src str, Spanned<LexerError<'src>>> {
        let raw = &self.src[start..end];
        match str::from_utf8(raw) {
            Ok(value) => Ok(value),
            Err(_) => self.emit_error(start, LexerError::InvalidUnicodeCharacter(raw)),
        }
    }

    fn emit(&self, start: usize, kind: TokenKind) -> LexerResult<'src> {
        let raw_content = self.slice_to_str(start, self.offset)?;

        Ok(Spanned::new(
            Token { raw_content, kind },
            Span {
                start,
                end: self.offset,
            },
        ))
    }

    #[inline(always)]
    fn advance_and_emit(&mut self, start: usize, len: usize, kind: TokenKind) -> LexerResult<'src> {
        self.advance_n(len);
        self.emit(start, kind)
    }

    fn emit_error<R>(
        &self,
        start: usize,
        error: LexerError<'src>,
    ) -> Result<R, Spanned<LexerError<'src>>> {
        Err(Spanned::new(
            error,
            Span {
                start,
                end: self.offset,
            },
        ))
    }
}
