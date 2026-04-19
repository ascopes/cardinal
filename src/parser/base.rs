use crate::errors::{SyntaxError, SyntaxResult};
use crate::lexer::Lexer;
use crate::spans::{Span, Spanned};
use crate::tokens::{Token, TokenKind};

/// Parses the outputs of a lexer into an abstract syntax tree.
pub struct Parser<'src> {
    lexer: Lexer<'src>,
    current_token: SyntaxResult<Spanned<Token<'src>>>,
}

impl<'src> Parser<'src> {
    pub fn new(mut lexer: Lexer<'src>) -> Self {
        let current_token = lexer.next_token();
        Self {
            lexer,
            current_token,
        }
    }

    pub fn parse(&mut self) -> SyntaxResult<Spanned<()>> {
        todo!();
    }

    #[inline(always)]
    pub(crate) fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    #[inline(always)]
    pub(crate) fn peek(&self) -> &SyntaxResult<Spanned<Token<'src>>> {
        &self.current_token
    }

    pub(crate) fn eat(
        &'src mut self,
        expected_kind: TokenKind,
    ) -> SyntaxResult<Spanned<Token<'src>>> {
        match &self.current_token {
            Ok(token) => {
                if token.value().kind() == expected_kind {
                    let cloned_token = token.clone();
                    self.advance();
                    Ok(cloned_token)
                } else {
                    let message = format!(
                        "expected token of type {:?} but got {:?}",
                        expected_kind,
                        token.value().kind()
                    );

                    Err(Spanned::new(
                        SyntaxError::UnexpectedToken {
                            message: message.into_boxed_str(),
                        },
                        token.span(),
                    ))
                }
            }
            Err(err) => Err(err.clone()),
        }
    }
}
