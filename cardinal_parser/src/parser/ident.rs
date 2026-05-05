use crate::ast::ident::{Ident, QualIdent, UnqualIdent};
use crate::parser::base::{Parser, ParserResult};
use crate::spans::{Span, Spanned};
use crate::tokens::TokenKind;

impl<'src> Parser<'src> {
    /// ```text
    /// ident      = unqual_ident | qual_ident ;
    /// qual_ident = IDENT , ( NAMESPACE_SEP , IDENT )+ ;
    /// ```
    pub(super) fn parse_ident(&mut self) -> ParserResult<Ident> {
        let mut token = self.eat(TokenKind::Ident)?;

        if self.peek()?.value().kind() != TokenKind::NamespaceSep {
            return self
                .parse_unqual_ident()
                .map(|ident| Spanned::new(Ident::Unqual(Box::new(ident.value())), ident.span()));
        }

        let mut names = vec![Spanned::new(
            self.extract_boxed_str(token.value()),
            token.span(),
        )];

        while self.peek()?.value().kind() == TokenKind::NamespaceSep {
            self.advance();
            token = self.eat(TokenKind::Ident)?;
            names.push(Spanned::new(
                self.extract_boxed_str(token.value()),
                token.span(),
            ));
        }

        let span = Span::of_spanned(&names.first().unwrap(), &names.last().unwrap());

        Ok(Spanned::new(
            Ident::Qual(Box::new(QualIdent { names })),
            span,
        ))
    }

    /// ```text
    /// unqual_ident = IDENT ;
    /// ```
    pub(super) fn parse_unqual_ident(&mut self) -> ParserResult<UnqualIdent> {
        self.eat(TokenKind::Ident).map(|token| {
            Spanned::new(
                UnqualIdent {
                    name: self.extract_boxed_str(token.value()),
                },
                token.span(),
            )
        })
    }
}
