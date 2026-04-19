use crate::ast::expr::Expr;
use crate::errors::SyntaxError;
use crate::lexer::Lexer;
use crate::spans::Spanned;
use crate::tokens::{Token, TokenKind};

/// Parses the outputs of a lexer into an abstract syntax tree.
pub struct Parser<'src> {
    lexer: Lexer<'src>,
    current_token: Result<Spanned<Token<'src>>, Spanned<SyntaxError>>,
}

impl<'src> Parser<'src> {
    pub fn new(mut lexer: Lexer<'src>) -> Self {
        let current_token = lexer.next_token();
        Self {
            lexer,
            current_token,
        }
    }

    pub fn parse(&mut self) -> Result<Spanned<Expr>, Spanned<SyntaxError>> {
        self.parse_expr()
    }

    #[inline(always)]
    pub(super) fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    // Mutability: this is always immutable in theory, but most operations require a mutable
    // borrow around the context of using this, so we keep it as a mutable borrow to keep the
    // borrow-checker happy.
    #[inline(always)]
    pub(super) fn peek(&mut self) -> Result<Spanned<Token<'src>>, Spanned<SyntaxError>> {
        self.current_token.clone()
    }

    pub(super) fn eat(
        &mut self,
        expected_kind: TokenKind,
    ) -> Result<Spanned<Token<'src>>, Spanned<SyntaxError>> {
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
