#[derive(Clone, Debug, PartialEq)]
pub struct Token<'src> {
    pub raw_content: &'src str,
    pub kind: TokenKind,
}

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
