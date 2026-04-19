use crate::ast::expr::*;
use crate::errors::SyntaxError;
use crate::parser::base::Parser;
use crate::spans::{Span, Spanned};
use crate::tokens::{Token, TokenKind};

impl<'src> Parser<'src> {
    /// ```text
    /// expr ::= assign_expr ;
    /// ```
    ///
    /// This is an alias to the top-level production when parsing expressions.
    #[inline]
    pub(super) fn parse_expr(&mut self) -> Result<Spanned<Expr>, Spanned<SyntaxError>> {
        self.parse_assign_expr()
    }

    /// ```text
    /// assign_expr ::= bool_or_expr , ASSIGN , assign_expr
    ///               | bool_or_expr , ADD_ASSIGN , assign_expr
    ///               | bool_or_expr , SUB_ASSIGN , assign_expr
    ///               | bool_or_expr , MUL_ASSIGN , assign_expr
    ///               | bool_or_expr , DIV_ASSIGN , assign_expr
    ///               | bool_or_expr , MOD_ASSIGN , assign_expr
    ///               | bool_or_expr , POW_ASSIGN , assign_expr
    ///               | bool_or_expr , BIT_AND_ASSIGN , assign_expr
    ///               | bool_or_expr , BIT_OR_ASSIGN , assign_expr
    ///               | bool_or_expr , BIT_XOR_ASSIGN , assign_expr
    ///               | bool_or_expr , BIT_SHL_ASSIGN , assign_expr
    ///               | bool_or_expr , BIT_SHR_ASSIGN , assign_expr
    ///               | bool_or_expr
    ///               ;
    /// ```
    fn parse_assign_expr(&mut self) -> Result<Spanned<Expr>, Spanned<SyntaxError>> {
        let left = self.parse_bool_or_expr()?;

        let op = match self.peek()?.value().kind() {
            TokenKind::Assign => None,
            TokenKind::AddAssign => Some(BinaryOp::Add),
            TokenKind::SubAssign => Some(BinaryOp::Sub),
            TokenKind::MulAssign => Some(BinaryOp::Mul),
            TokenKind::DivAssign => Some(BinaryOp::Div),
            TokenKind::ModAssign => Some(BinaryOp::Mod),
            TokenKind::PowAssign => Some(BinaryOp::Pow),
            TokenKind::BitAndAssign => Some(BinaryOp::BitAnd),
            TokenKind::BitOrAssign => Some(BinaryOp::BitOr),
            TokenKind::BitXorAssign => Some(BinaryOp::BitXor),
            TokenKind::BitShlAssign => Some(BinaryOp::BitShl),
            TokenKind::BitShrAssign => Some(BinaryOp::BitShr),
            _ => return Ok(left),
        };
        self.advance();

        // Purposely recursive here, to force right-associativity.
        // `x = y = z = a` is parsed as `(x = (y = (z = a)))`
        let right = self.parse_assign_expr()?;

        let span = Span::of_spanned(&left, &right);

        Ok(Spanned::new(
            Expr::Assign(Box::new(AssignExpr { left, op, right })),
            span,
        ))
    }

    /// ```text
    /// bool_or_expr ::= bool_and_expr , BIT_OR , bool_or_expr
    ///                | bool_and_expr
    ///                ;
    /// ```
    fn parse_bool_or_expr(&mut self) -> Result<Spanned<Expr>, Spanned<SyntaxError>> {
        self.parse_binary_op_left_assoc(
            |token| match token.kind() {
                TokenKind::BoolOr => Some(BinaryOp::BoolOr),
                _ => None,
            },
            Self::parse_bool_and_expr,
        )
    }

    /// ```text
    /// bool_and_expr ::= eq_expr , BIT_AND , bool_and_expr
    ///                 | eq_expr
    ///                 ;
    /// ```
    fn parse_bool_and_expr(&mut self) -> Result<Spanned<Expr>, Spanned<SyntaxError>> {
        self.parse_binary_op_left_assoc(
            |token| match token.kind() {
                TokenKind::BoolAnd => Some(BinaryOp::BoolAnd),
                _ => None,
            },
            Self::parse_eq_expr,
        )
    }

    /// ```text
    /// eq_expr ::= comp_expr , EQ , eq_expr
    ///           | comp_expr , NEQ , eq_expr
    ///           | comp_expr
    ///           ;
    /// ```
    fn parse_eq_expr(&mut self) -> Result<Spanned<Expr>, Spanned<SyntaxError>> {
        self.parse_binary_op_left_assoc(
            |token| match token.kind() {
                TokenKind::Eq => Some(BinaryOp::Eq),
                TokenKind::Neq => Some(BinaryOp::Neq),
                _ => None,
            },
            Self::parse_comp_expr,
        )
    }

    /// ```text
    /// comp_expr ::= bitshift_expr , LT , comp_expr
    ///             | bitshift_expr , LTEQ , comp_expr
    ///             | bitshift_expr , GT , comp_expr
    ///             | bitshift_expr , GTEQ , comp_expr
    ///             | bitshift_expr
    ///             ;
    /// ```
    fn parse_comp_expr(&mut self) -> Result<Spanned<Expr>, Spanned<SyntaxError>> {
        self.parse_binary_op_left_assoc(
            |token| match token.kind() {
                TokenKind::Lt => Some(BinaryOp::Lt),
                TokenKind::LtEq => Some(BinaryOp::LtEq),
                TokenKind::Gt => Some(BinaryOp::Gt),
                TokenKind::GtEq => Some(BinaryOp::GtEq),
                _ => None,
            },
            Self::parse_bitshift_expr,
        )
    }

    /// ```text
    /// bitshift_expr ::= sum_expr , BIT_SHL , bitshift_expr
    ///                 | sum_expr , BIT_SHR , bitshift_expr
    ///                 | sum_expr
    ///                 ;
    /// ```
    fn parse_bitshift_expr(&mut self) -> Result<Spanned<Expr>, Spanned<SyntaxError>> {
        self.parse_binary_op_left_assoc(
            |token| match token.kind() {
                TokenKind::BitShl => Some(BinaryOp::BitShl),
                TokenKind::BitShr => Some(BinaryOp::BitShr),
                _ => None,
            },
            Self::parse_sum_expr,
        )
    }

    /// ```text
    /// sum_expr ::= factor_expr , ADD , sum_expr
    ///            | factor_expr , SUB , sum_expr
    ///            | factor_expr
    ///            ;
    /// ```
    fn parse_sum_expr(&mut self) -> Result<Spanned<Expr>, Spanned<SyntaxError>> {
        self.parse_binary_op_left_assoc(
            |token| match token.kind() {
                TokenKind::Add => Some(BinaryOp::Add),
                TokenKind::Sub => Some(BinaryOp::Sub),
                _ => None,
            },
            Self::parse_factor_expr,
        )
    }

    /// ```text
    /// factor_expr ::= unary_expr , MUL , factor_expr
    ///               | unary_expr , SUB , factor_expr
    ///               | unary_expr
    ///               ;
    /// ```
    fn parse_factor_expr(&mut self) -> Result<Spanned<Expr>, Spanned<SyntaxError>> {
        self.parse_binary_op_left_assoc(
            |token| match token.kind() {
                TokenKind::Mul => Some(BinaryOp::Mul),
                TokenKind::Div => Some(BinaryOp::Div),
                TokenKind::Mod => Some(BinaryOp::Mod),
                _ => None,
            },
            Self::parse_unary_expr,
        )
    }

    /// ```text
    /// unary_expr ::= ADD , unary_expr
    ///              | SUB , unary_expr
    ///              | BIT_NOT , unary_expr
    ///              | BOOL_NOT , unary_expr
    ///              | pow_expr
    ///              ;
    /// ```
    fn parse_unary_expr(&mut self) -> Result<Spanned<Expr>, Spanned<SyntaxError>> {
        let first = self.peek()?;
        let op = match first.value().kind() {
            TokenKind::Add => UnaryOp::Plus,
            TokenKind::Sub => UnaryOp::Minus,
            TokenKind::BitNot => UnaryOp::BoolNot,
            TokenKind::BoolNot => UnaryOp::BitNot,
            _ => return self.parse_pow_expr(),
        };
        self.advance();

        let value = self.parse_unary_expr()?;
        let span = Span::of_spanned(&first, &value);

        Ok(Spanned::new(
            Expr::Unary(Box::new(UnaryExpr { op, value })),
            span,
        ))
    }

    /// ```text
    /// pow_expr ::= pow_expr , POW , primary_expr
    ///            | primary_expr
    ///            ;
    /// ```
    fn parse_pow_expr(&mut self) -> Result<Spanned<Expr>, Spanned<SyntaxError>> {
        let left = self.parse_primary_expr()?;

        let op = match self.peek()?.value().kind() {
            TokenKind::Pow => BinaryOp::Pow,
            _ => return Ok(left),
        };
        self.advance();

        // Purposely recursive here, to force right-associativity.
        // In maths, we always say `x ** y ** z` is `(x ** (y ** z))`. This differs to
        // most of the expr grammar here, so we treat it as an edge case and do not
        // wrap it in a utility handler helper.
        let right = self.parse_pow_expr()?;

        let span = Span::of_spanned(&left, &right);

        Ok(Spanned::new(
            Expr::Binary(Box::new(BinaryExpr { left, op, right })),
            span,
        ))
    }

    /// ```text
    /// primary_expr ::= atom , ( member_access_expr | index_expr | func_call_expr )*
    ///                ;
    /// ```
    fn parse_primary_expr(&mut self) -> Result<Spanned<Expr>, Spanned<SyntaxError>> {
        let mut expr = self.parse_atom()?;

        // Consume chained calls and selectors
        loop {
            expr = match self.peek()?.value().kind() {
                TokenKind::Period => self.parse_member_access_expr(expr)?,
                TokenKind::LeftBracket => self.parse_index_expr(expr)?,
                TokenKind::LeftParen => self.parse_func_call_expr(expr)?,
                _ => break,
            }
        }

        Ok(expr)
    }

    /// ```text
    /// member_access_expr ::= PERIOD , ident ;
    /// ```
    fn parse_member_access_expr(
        &mut self,
        owner: Spanned<Expr>,
    ) -> Result<Spanned<Expr>, Spanned<SyntaxError>> {
        debug_assert_eq!(self.peek()?.value().kind(), TokenKind::Period);

        self.advance();
        let member = self.parse_ident()?;
        let span = Span::of_spanned(&owner, &member);
        Ok(Spanned::new(
            Expr::MemberAccess(Box::new(MemberAccessExpr { owner, member })),
            span,
        ))
    }

    /// ```text
    /// index_expr ::= LEFT_BRACKET , expr , RIGHT_BRACKET ;
    /// ```
    fn parse_index_expr(
        &mut self,
        owner: Spanned<Expr>,
    ) -> Result<Spanned<Expr>, Spanned<SyntaxError>> {
        debug_assert_eq!(self.peek()?.value().kind(), TokenKind::LeftBracket);

        self.advance();
        let index = self.parse_expr()?;
        let right_sq = self.eat(TokenKind::RightBracket)?;
        let span = Span::of_spanned(&owner, &right_sq);
        Ok(Spanned::new(
            Expr::Index(Box::new(IndexExpr { owner, index })),
            span,
        ))
    }

    /// ```text
    /// func_call_expr ::= LEFT_PAREN , arg_list , RIGHT_PAREN ;
    /// arg_list       ::= expr , ( COMMA , expr )* ;
    /// ```
    fn parse_func_call_expr(
        &mut self,
        name: Spanned<Expr>,
    ) -> Result<Spanned<Expr>, Spanned<SyntaxError>> {
        debug_assert_eq!(self.peek()?.value().kind(), TokenKind::LeftParen);

        let left_paren_span = self.eat(TokenKind::LeftParen)?.span();
        let mut arguments = Vec::<Spanned<Expr>>::new();

        // Allow zero or more arguments, which are expressions.
        while !matches!(self.peek()?.value().kind(), TokenKind::RightParen) {
            arguments.push(self.parse_expr()?);

            if matches!(self.peek()?.value().kind(), TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        let right_paren = self.eat(TokenKind::RightParen)?;
        let entire_span = Span::of_spanned(&name, &right_paren);
        let argument_span = Span::of(left_paren_span, right_paren.span());

        Ok(Spanned::new(
            Expr::FuncCall(Box::new(FuncCallExpr {
                name,
                arguments: Spanned::new(arguments.into_boxed_slice(), argument_span),
            })),
            entire_span,
        ))
    }

    fn parse_atom(&mut self) -> Result<Spanned<Expr>, Spanned<SyntaxError>> {
        let first = self.peek()?;

        match first.value().kind() {
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.eat(TokenKind::RightParen)?;
                Ok(expr)
            }
            // TODO(ascopes): parse identifier paths here, i.e. foo::bar::baz
            TokenKind::Ident => {
                let ident = self.parse_ident()?;
                Ok(Spanned::new(
                    Expr::Ident(Box::new(ident.value())),
                    ident.span(),
                ))
            }
            TokenKind::BoolLit(value) => {
                self.advance();
                Ok(Spanned::new(
                    Expr::Bool(Box::new(BoolLitExpr { value })),
                    first.span(),
                ))
            }
            TokenKind::IntLit(value) => {
                self.advance();
                Ok(Spanned::new(
                    Expr::Int(Box::new(IntLitExpr { value })),
                    first.span(),
                ))
            }
            TokenKind::FloatLit(value) => {
                self.advance();
                Ok(Spanned::new(
                    Expr::Float(Box::new(FloatLitExpr { value })),
                    first.span(),
                ))
            }
            TokenKind::StrLit => {
                // TODO: implement string literal parsing in new file.
                todo!()
            }
            _ => Err(Spanned::new(
                SyntaxError::UnexpectedToken {
                    message: format!(
                        "expected atom (literal, identifier, expression in parenthesis), got {:?}",
                        first.value().kind()
                    )
                    .into_boxed_str(),
                },
                first.span(),
            )),
        }
    }

    /// Shortcut to parse a binary operator with left-associativity (e.g. `1 + 2 + 3 + 4`
    /// becomes `(((1 + 2) + 3) + 4)`).
    ///
    /// This flattens the logic down to an iterative approach to avoid growing the stack
    /// excessively most of the time.
    fn parse_binary_op_left_assoc<OpFn, ParserFn>(
        &mut self,
        op_fn: OpFn,
        parser_fn: ParserFn,
    ) -> Result<Spanned<Expr>, Spanned<SyntaxError>>
    where
        OpFn: Fn(Token) -> Option<BinaryOp>,
        ParserFn: Fn(&mut Self) -> Result<Spanned<Expr>, Spanned<SyntaxError>>,
    {
        let mut root = parser_fn(self)?;

        loop {
            if let Some(op) = op_fn(self.peek()?.value()) {
                self.advance();
                let right = parser_fn(self)?;

                let span = Span::of_spanned(&root, &right);

                root = Spanned::new(
                    Expr::Binary(Box::from(BinaryExpr {
                        left: root,
                        op,
                        right,
                    })),
                    span,
                );
            } else {
                break;
            }
        }

        Ok(root)
    }
}
