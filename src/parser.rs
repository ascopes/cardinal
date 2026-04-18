use crate::errors::{SyntaxError, SyntaxResult};
use crate::lexer::Lexer;
use crate::spans::Spanned;
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
    fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    #[inline(always)]
    fn peek(&self) -> &SyntaxResult<Spanned<Token>> {
        &self.current_token
    }

    fn eat(
        &'src mut self,
        expected_kind: TokenKind,
        error_message: &str,
    ) -> SyntaxResult<Spanned<Token<'src>>> {
        match &self.current_token {
            Ok(token) => {
                if token.value().kind() == expected_kind {
                    Ok(token.clone())
                } else {
                    Err(Spanned::new(
                        SyntaxError::UnexpectedToken {
                            message: Box::from(error_message),
                        },
                        token.span(),
                    ))
                }
            }

            Err(err) => Err(err.clone()),
        }
    }
}
