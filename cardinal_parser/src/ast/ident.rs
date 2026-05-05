use crate::spans::Spanned;

#[derive(Clone, Debug)]
pub enum Ident {
    Unqual(Box<UnqualIdent>),
    Qual(Box<QualIdent>),
}

#[derive(Clone, Debug)]
pub struct QualIdent {
    // TODO: do we need to span each box or is it useless for most
    //  diagnostic purposes?
    pub names: Vec<Spanned<Box<str>>>,
}

#[derive(Clone, Debug)]
pub struct UnqualIdent {
    pub name: Box<str>,
}
