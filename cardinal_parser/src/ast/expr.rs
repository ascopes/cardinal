use crate::ast::ident::Ident;
use crate::spans::Spanned;

#[derive(Clone, Debug)]
pub enum Expr {
    Bool(Box<BoolLitExpr>),
    Int(Box<IntLitExpr>),
    Float(Box<FloatLitExpr>),
    Str(Box<StrLitExpr>),
    Ident(Box<Ident>),
    Assign(Box<AssignExpr>),
    Binary(Box<BinaryExpr>),
    Unary(Box<UnaryExpr>),
    MemberAccess(Box<MemberAccessExpr>),
    Index(Box<IndexExpr>),
    FuncCall(Box<FuncCallExpr>),
}

#[derive(Clone, Debug)]
pub struct BoolLitExpr {
    pub value: bool,
}

#[derive(Clone, Debug)]
pub struct IntLitExpr {
    pub value: u64,
}

#[derive(Clone, Debug)]
pub struct FloatLitExpr {
    pub value: f64,
}

#[derive(Clone, Debug)]
pub struct StrLitExpr {
    pub value: Box<str>,
}

#[derive(Clone, Debug)]
pub struct AssignExpr {
    pub left: Spanned<Expr>,
    /// Set if the assignment operation is composite (e.g. `foo += bar` implies an op of `Add`).
    /// For all other purposes, this will remain unset.
    pub op: Option<BinaryOp>,
    pub right: Spanned<Expr>,
}

#[derive(Clone, Debug)]
pub struct BinaryExpr {
    pub left: Spanned<Expr>,
    pub op: BinaryOp,
    pub right: Spanned<Expr>,
}

#[derive(Clone, Copy, Debug)]
pub enum BinaryOp {
    // Arithmetic.
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,

    // Bitwise.
    BitAnd,
    BitOr,
    BitXor,
    BitShl,
    BitShr,

    // Boolean.
    BoolAnd,
    BoolOr,

    // Comparative.
    Eq,
    Neq,
    Gt,
    GtEq,
    Lt,
    LtEq,
}

#[derive(Clone, Debug)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub value: Spanned<Expr>,
}

#[derive(Clone, Copy, Debug)]
pub enum UnaryOp {
    // Arithmetic.
    Plus,
    Minus,

    // Bitwise.
    BitNot,

    // Boolean.
    BoolNot,
}

#[derive(Clone, Debug)]
pub struct MemberAccessExpr {
    pub owner: Spanned<Expr>,
    pub member: Spanned<Ident>,
}

#[derive(Clone, Debug)]
pub struct IndexExpr {
    pub owner: Spanned<Expr>,
    pub index: Spanned<Expr>,
}

#[derive(Clone, Debug)]
pub struct FuncCallExpr {
    pub name: Spanned<Expr>,
    pub arguments: Spanned<Box<[Spanned<Expr>]>>,
}
