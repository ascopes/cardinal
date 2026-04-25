use crate::ast::ident::{Ident, QualIdent, SimpleIdent};
use crate::errors::SyntaxError;
use crate::parser::base::Parser;
use crate::spans::{Span, Spanned};
use crate::tokens::TokenKind;

impl<'src> Parser<'src> {
    /// ```text
    /// ident = IDENTIFIER , ( NAMESPACE_SEP , IDENTIFIER )+
    ///       | IDENTIFIER
    ///       ;
    /// ```
    pub(super) fn parse_ident(&mut self) -> Result<Spanned<Ident>, Spanned<SyntaxError>> {
        let mut token = self.eat(TokenKind::Ident)?;

        if self.peek()?.value().kind() != TokenKind::NamespaceSep {
            return Ok(Spanned::new(
                Ident::Simple(Box::new(SimpleIdent {
                    name: self.extract_boxed_str(token.value()),
                })),
                token.span(),
            ));
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
}
