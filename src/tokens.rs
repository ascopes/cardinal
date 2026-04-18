/// Type that holds details about a token that was lifted from a source file during
/// lexical analysis.
///
/// Each token holds the raw content of the token and the details of the variant. The
/// raw content of the token is a reference to a parsed slice within the original source
/// file.
#[derive(Clone, Debug, PartialEq)]
pub struct Token<'src> {
    /// Raw UTF-8 encoded contents of the token.
    raw_content: &'src str,

    /// Details about the kind of token.
    kind: TokenKind,
}

impl<'src> Token<'src> {
    #[inline(always)]
    pub fn new(raw_content: &'src str, kind: TokenKind) -> Self {
        Self { raw_content, kind }
    }

    #[inline(always)]
    pub fn raw_content(&self) -> &'src str {
        self.raw_content
    }

    #[inline(always)]
    pub fn kind(&self) -> TokenKind {
        self.kind
    }
}

/// Describes the variant of a token.
///
/// Some simple opaque types can be included in this variant, but for the most part,
/// it is down to a parser to actually parse the contents of the token.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenKind {
    /* Special markers */
    Eof,

    /* Comments */
    SingleLineComment,
    MultiLineComment,

    /* Literals */
    BoolLit(bool),
    StrLit,
    IntLit(u64),
    FloatLit(f64),
    Ident,

    Fn,
    If,
    Else,
    While,
    For,
    Return,
    Break,
    Continue,
    Let,

    /* Arithmetic operators */
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    PowAssign,

    /* Binary operators */
    BitNot,
    BitAnd,
    BitOr,
    BitXor,
    BitShl,
    BitShr,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    BitShlAssign,
    BitShrAssign,

    /* Boolean operators */
    BoolNot,
    BoolAnd,
    BoolOr,

    /* Comparative operators */
    Eq,
    Neq,
    Lt,
    LtEq,
    Gt,
    GtEq,

    /* Special operators */
    Assign,

    /* Blocks */
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,

    /* Other tokens */
    Semi,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_constructed_as_expected() {
        // Given
        let raw_content = "foobarbaz";
        let kind = TokenKind::Ident;

        // When
        let token = Token::new(raw_content, kind);

        // Then
        assert_eq!(token.raw_content(), raw_content, "raw_content");
        assert_eq!(token.kind(), kind, "kind");
    }
}
