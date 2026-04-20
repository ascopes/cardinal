use crate::ast::ident::Ident;
use crate::errors::SyntaxError;
use crate::parser::base::Parser;
use crate::spans::Spanned;
use crate::tokens::TokenKind;

impl<'src> Parser<'src> {
    /// ```text
    /// identifier ::= IDENTIFIER ;
    /// ```
    pub(super) fn parse_ident(&mut self) -> Result<Spanned<Ident>, Spanned<SyntaxError>> {
        let current = self.peek()?;
        if current.value().kind() == TokenKind::Ident {
            self.advance();
            Ok(Spanned::new(
                Ident {
                    value: current.value().raw_content().to_string(),
                },
                current.span(),
            ))
        } else {
            let err = Spanned::new(
                SyntaxError::UnexpectedToken {
                    message: Box::from("expected identifier"),
                },
                current.span(),
            );
            Err(err)
        }
    }
}
