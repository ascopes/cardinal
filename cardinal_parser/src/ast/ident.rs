use crate::spans::Spanned;

#[derive(Clone, Debug)]
pub enum Ident {
    Simple(Box<SimpleIdent>),
    Qual(Box<QualIdent>),
}

#[derive(Clone, Debug)]
pub struct SimpleIdent {
    pub name: Box<str>,
}

#[derive(Clone, Debug)]
pub struct QualIdent {
    pub names: Vec<Spanned<Box<str>>>,
}
