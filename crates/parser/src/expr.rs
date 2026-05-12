use super::*;

#[path = "operators.rs"]
mod operators;

impl Parser {
    pub(super) fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_expr_with_hint(None)
    }

    pub(super) fn parse_callable_expr(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_expr()?;
        match expr {
            Expr::Call { .. } => Ok(expr),
            Expr::FunctionLiteral { .. } => {
                self.skip_semicolons();
                self.expect_punctuation("`(`", |kind| matches!(kind, TokenKind::LParen))?;
                let args = self.parse_call_args()?;
                self.expect_punctuation("`)`", |kind| matches!(kind, TokenKind::RParen))?;
                Ok(Expr::Call {
                    callee: Box::new(expr),
                    type_args: Vec::new(),
                    args,
                })
            }
            other => Err(ParseError::UnexpectedToken {
                expected: "call expression".into(),
                found: format!("{other:?}"),
            }),
        }
    }

    fn parse_expr_with_hint(
        &mut self,
        literal_type_hint: Option<&str>,
    ) -> Result<Expr, ParseError> {
        self.parse_or_expr_with_hint(literal_type_hint)
    }

    fn parse_or_expr_with_hint(
        &mut self,
        literal_type_hint: Option<&str>,
    ) -> Result<Expr, ParseError> {
        let mut expr = self.parse_and_expr_with_hint(literal_type_hint)?;
        while self.check(|kind| matches!(kind, TokenKind::OrOr)) {
            self.bump();
            let right = self.parse_and_expr_with_hint(None)?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::Or,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_and_expr_with_hint(
        &mut self,
        literal_type_hint: Option<&str>,
    ) -> Result<Expr, ParseError> {
        let mut expr = self.parse_comparison_expr_with_hint(literal_type_hint)?;
        while self.check(|kind| matches!(kind, TokenKind::AndAnd)) {
            self.bump();
            let right = self.parse_comparison_expr_with_hint(None)?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::And,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_comparison_expr_with_hint(
        &mut self,
        literal_type_hint: Option<&str>,
    ) -> Result<Expr, ParseError> {
        let mut expr = self.parse_add_expr_with_hint(literal_type_hint)?;
        while let Some(op) = self.parse_comparison_op() {
            let right = self.parse_add_expr_with_hint(None)?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_add_expr_with_hint(
        &mut self,
        literal_type_hint: Option<&str>,
    ) -> Result<Expr, ParseError> {
        let mut expr = self.parse_multiply_expr_with_hint(literal_type_hint)?;
        while let Some(op) = self.parse_add_op() {
            let right = self.parse_multiply_expr_with_hint(None)?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_multiply_expr_with_hint(
        &mut self,
        literal_type_hint: Option<&str>,
    ) -> Result<Expr, ParseError> {
        let mut expr = self.parse_unary_expr_with_hint(literal_type_hint)?;
        while let Some(op) = self.parse_multiply_op() {
            let right = self.parse_unary_expr_with_hint(None)?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_unary_expr_with_hint(
        &mut self,
        literal_type_hint: Option<&str>,
    ) -> Result<Expr, ParseError> {
        if self.check(|kind| matches!(kind, TokenKind::Bang)) {
            self.bump();
            return Ok(Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(self.parse_unary_expr_with_hint(None)?),
            });
        }

        if self.check(|kind| matches!(kind, TokenKind::Minus)) {
            self.bump();
            return Ok(Expr::Unary {
                op: UnaryOp::Negate,
                expr: Box::new(self.parse_unary_expr_with_hint(None)?),
            });
        }

        if self.check(|kind| matches!(kind, TokenKind::Caret)) {
            self.bump();
            return Ok(Expr::Unary {
                op: UnaryOp::BitNot,
                expr: Box::new(self.parse_unary_expr_with_hint(None)?),
            });
        }

        if self.check(|kind| matches!(kind, TokenKind::BitAnd)) {
            self.bump();
            return Ok(Expr::Unary {
                op: UnaryOp::AddressOf,
                expr: Box::new(self.parse_unary_expr_with_hint(None)?),
            });
        }

        if self.check(|kind| matches!(kind, TokenKind::Star)) {
            self.bump();
            return Ok(Expr::Unary {
                op: UnaryOp::Deref,
                expr: Box::new(self.parse_unary_expr_with_hint(None)?),
            });
        }

        if self.check(|kind| matches!(kind, TokenKind::Arrow)) {
            self.bump();
            return Ok(Expr::Unary {
                op: UnaryOp::Receive,
                expr: Box::new(self.parse_unary_expr_with_hint(None)?),
            });
        }

        self.parse_postfix_expr_with_hint(literal_type_hint)
    }

    fn parse_postfix_expr_with_hint(
        &mut self,
        literal_type_hint: Option<&str>,
    ) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary_with_hint(literal_type_hint)?;
        loop {
            if matches!(expr, Expr::Ident(_) | Expr::Selector { .. })
                && self.check(|kind| matches!(kind, TokenKind::LBracket))
                && self.peek_is_type_arg_call()
            {
                self.bump();
                let type_args = self.parse_type_arg_list()?;
                self.expect_punctuation("`]`", |kind| matches!(kind, TokenKind::RBracket))?;
                self.expect_punctuation("`(`", |kind| matches!(kind, TokenKind::LParen))?;
                let args = self.parse_call_args()?;
                self.expect_punctuation("`)`", |kind| matches!(kind, TokenKind::RParen))?;
                expr = Expr::Call {
                    callee: Box::new(expr),
                    type_args,
                    args,
                };
                continue;
            }

            if matches!(expr, Expr::Ident(_) | Expr::Selector { .. })
                && self.check(|kind| matches!(kind, TokenKind::LBracket))
                && self.peek_is_generic_struct_literal()
            {
                let Some(base_name) = self.expr_type_name(&expr) else {
                    break;
                };
                self.bump();
                let type_args = self.parse_type_arg_list()?;
                self.expect_punctuation("`]`", |kind| matches!(kind, TokenKind::RBracket))?;
                expr =
                    self.parse_struct_literal(format!("{base_name}[{}]", type_args.join(",")))?;
                continue;
            }

            if let Some(type_name) = self.expr_type_name(&expr) {
                if self.looks_like_struct_literal() {
                    expr = self.parse_struct_literal(type_name)?;
                    continue;
                }
            }

            if self.check(|kind| matches!(kind, TokenKind::Dot)) {
                self.bump();
                if self.check(|kind| matches!(kind, TokenKind::LParen)) {
                    if self
                        .tokens
                        .get(self.cursor + 1)
                        .is_some_and(|t| matches!(t.kind, TokenKind::Type))
                    {
                        self.cursor -= 1;
                        break;
                    }
                    self.bump();
                    let asserted_type = self.parse_type_name()?;
                    self.expect_punctuation("`)`", |kind| matches!(kind, TokenKind::RParen))?;
                    expr = Expr::TypeAssert {
                        expr: Box::new(expr),
                        asserted_type,
                    };
                    continue;
                }
                let field = self.expect_ident()?;
                expr = Expr::Selector {
                    receiver: Box::new(expr),
                    field,
                };
                continue;
            }

            if self.check(|kind| matches!(kind, TokenKind::LParen)) {
                self.bump();
                if let Expr::Ident(name) = &expr {
                    if name == "new" {
                        let type_name = self.parse_type_name()?;
                        self.expect_punctuation("`)`", |kind| matches!(kind, TokenKind::RParen))?;
                        expr = Expr::New { type_name };
                        continue;
                    }
                    if name == "make" {
                        let type_name = self.parse_type_name()?;
                        let args = if self.check(|kind| matches!(kind, TokenKind::Comma)) {
                            self.bump();
                            self.parse_call_args()?
                        } else {
                            Vec::new()
                        };
                        self.expect_punctuation("`)`", |kind| matches!(kind, TokenKind::RParen))?;
                        expr = Expr::Make { type_name, args };
                        continue;
                    }
                }
                let args = self.parse_call_args()?;
                self.expect_punctuation("`)`", |kind| matches!(kind, TokenKind::RParen))?;
                expr = Expr::Call {
                    callee: Box::new(expr),
                    type_args: Vec::new(),
                    args,
                };
                continue;
            }

            if self.check(|kind| matches!(kind, TokenKind::LBracket)) {
                self.bump();
                if self.check(|kind| matches!(kind, TokenKind::Colon)) {
                    self.bump();
                    let high = if self.check(|kind| matches!(kind, TokenKind::RBracket)) {
                        None
                    } else {
                        Some(Box::new(self.parse_expr()?))
                    };
                    self.expect_punctuation("`]`", |kind| matches!(kind, TokenKind::RBracket))?;
                    expr = Expr::SliceExpr {
                        target: Box::new(expr),
                        low: None,
                        high,
                    };
                } else {
                    let first = self.parse_expr()?;
                    if self.check(|kind| matches!(kind, TokenKind::Colon)) {
                        self.bump();
                        let high = if self.check(|kind| matches!(kind, TokenKind::RBracket)) {
                            None
                        } else {
                            Some(Box::new(self.parse_expr()?))
                        };
                        self.expect_punctuation("`]`", |kind| matches!(kind, TokenKind::RBracket))?;
                        expr = Expr::SliceExpr {
                            target: Box::new(expr),
                            low: Some(Box::new(first)),
                            high,
                        };
                    } else {
                        self.expect_punctuation("`]`", |kind| matches!(kind, TokenKind::RBracket))?;
                        expr = Expr::Index {
                            target: Box::new(expr),
                            index: Box::new(first),
                        };
                    }
                }
                continue;
            }

            break;
        }
        Ok(expr)
    }

    fn expr_type_name(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Ident(name) => Some(name.clone()),
            Expr::Selector { receiver, field } => match &**receiver {
                Expr::Ident(package) => Some(format!("{package}.{field}")),
                _ => None,
            },
            _ => None,
        }
    }

    fn peek_is_type_arg_call(&self) -> bool {
        let mut lookahead = Parser {
            tokens: self.tokens.clone(),
            cursor: self.cursor + 1,
            allow_empty_struct_literal: self.allow_empty_struct_literal,
            allow_named_struct_literal: self.allow_named_struct_literal,
            source_file_spans: SourceFileSpans::empty(),
            stmt_spans: Vec::new(),
        };
        let first_arg_start = lookahead.cursor;
        if !lookahead.starts_type_name() {
            return false;
        }
        if lookahead.parse_type_name().is_err() {
            return false;
        }
        let first_arg_end = lookahead.cursor;
        let mut has_multiple_args = false;
        while lookahead.check(|kind| matches!(kind, TokenKind::Comma)) {
            has_multiple_args = true;
            lookahead.bump();
            if !lookahead.starts_type_name() || lookahead.parse_type_name().is_err() {
                return false;
            }
        }
        if !lookahead.check(|kind| matches!(kind, TokenKind::RBracket)) {
            return false;
        }
        lookahead.bump();
        lookahead.skip_semicolons();
        lookahead.check(|kind| matches!(kind, TokenKind::LParen))
            && (has_multiple_args
                || !self.type_arg_tokens_are_ambiguous(
                    &lookahead.tokens[first_arg_start..first_arg_end],
                ))
    }

    fn peek_is_generic_struct_literal(&self) -> bool {
        let mut lookahead = Parser {
            tokens: self.tokens.clone(),
            cursor: self.cursor + 1,
            allow_empty_struct_literal: self.allow_empty_struct_literal,
            allow_named_struct_literal: self.allow_named_struct_literal,
            source_file_spans: SourceFileSpans::empty(),
            stmt_spans: Vec::new(),
        };
        if !lookahead.starts_type_name() || lookahead.parse_type_name().is_err() {
            return false;
        }
        while lookahead.check(|kind| matches!(kind, TokenKind::Comma)) {
            lookahead.bump();
            if !lookahead.starts_type_name() || lookahead.parse_type_name().is_err() {
                return false;
            }
        }
        if !lookahead.check(|kind| matches!(kind, TokenKind::RBracket)) {
            return false;
        }
        lookahead.bump();
        lookahead.looks_like_struct_literal()
    }

    fn parse_type_arg_list(&mut self) -> Result<Vec<String>, ParseError> {
        let mut type_args = vec![self.parse_type_name()?];
        while self.check(|kind| matches!(kind, TokenKind::Comma)) {
            self.bump();
            type_args.push(self.parse_type_name()?);
        }
        Ok(type_args)
    }

    fn type_arg_tokens_are_ambiguous(&self, tokens: &[Token]) -> bool {
        !tokens.is_empty()
            && tokens
                .iter()
                .all(|token| matches!(token.kind, TokenKind::Ident(_) | TokenKind::Dot))
    }

    fn parse_call_args(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();
        while self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
            self.bump();
        }
        if self.check(|kind| matches!(kind, TokenKind::RParen)) {
            return Ok(args);
        }

        loop {
            let expr = self.parse_expr()?;
            if self.check(|kind| matches!(kind, TokenKind::Ellipsis)) {
                self.bump();
                args.push(Expr::Spread {
                    expr: Box::new(expr),
                });
            } else {
                args.push(expr);
            }
            while self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                self.bump();
            }
            if !self.check(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
            self.bump();
            while self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                self.bump();
            }
            if self.check(|kind| matches!(kind, TokenKind::RParen)) {
                break;
            }
        }

        Ok(args)
    }

    fn parse_primary_with_hint(
        &mut self,
        literal_type_hint: Option<&str>,
    ) -> Result<Expr, ParseError> {
        let token = self.current_token()?;
        match &token.kind {
            TokenKind::LBrace => {
                if let Some(type_name) = literal_type_hint {
                    return self.parse_struct_literal(type_name.to_string());
                }
                Err(ParseError::UnexpectedToken {
                    expected: "expression".into(),
                    found: describe_token(&token.kind),
                })
            }
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.bump();
                if self.looks_like_struct_literal() {
                    return self.parse_struct_literal(name);
                }
                Ok(Expr::Ident(name))
            }
            TokenKind::True => {
                self.bump();
                Ok(Expr::BoolLiteral(true))
            }
            TokenKind::False => {
                self.bump();
                Ok(Expr::BoolLiteral(false))
            }
            TokenKind::Nil => {
                self.bump();
                Ok(Expr::NilLiteral)
            }
            TokenKind::Int(value) => {
                let value = *value;
                self.bump();
                Ok(Expr::IntLiteral(value))
            }
            TokenKind::Float(bits) => {
                let bits = *bits;
                self.bump();
                Ok(Expr::FloatLiteral(bits))
            }
            TokenKind::String(value) => {
                let value = value.clone();
                self.bump();
                Ok(Expr::StringLiteral(value))
            }
            TokenKind::LParen => {
                self.bump();
                let expr = self.parse_expr_with_hint(None)?;
                self.expect_punctuation("`)`", |kind| matches!(kind, TokenKind::RParen))?;
                Ok(expr)
            }
            TokenKind::LBracket => self.parse_collection_literal(),
            TokenKind::Map => self.parse_map_literal(),
            TokenKind::Func => self.parse_function_literal(),
            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".into(),
                found: describe_token(&token.kind),
            }),
        }
    }

    fn parse_function_literal(&mut self) -> Result<Expr, ParseError> {
        self.expect_keyword("`func`", TokenKind::Func)?;
        self.expect_punctuation("`(`", |kind| matches!(kind, TokenKind::LParen))?;
        let params = self.parse_parameter_list(true)?;
        self.expect_punctuation("`)`", |kind| matches!(kind, TokenKind::RParen))?;
        let result_types = self.parse_result_types()?;
        let body = self.parse_block()?;
        Ok(Expr::FunctionLiteral {
            params,
            result_types,
            body,
        })
    }

    fn parse_struct_literal(&mut self, type_name: String) -> Result<Expr, ParseError> {
        self.expect_punctuation("`{`", |kind| matches!(kind, TokenKind::LBrace))?;
        let mut fields = Vec::new();
        if self.check(|kind| matches!(kind, TokenKind::RBrace)) {
            self.bump();
            return Ok(Expr::StructLiteral { type_name, fields });
        }
        let field_type_hints = struct_field_type_hints(&type_name);

        let positional = !self.check(|kind| matches!(kind, TokenKind::Ident(_)))
            || !matches!(
                self.tokens.get(self.cursor + 1).map(|t| &t.kind),
                Some(TokenKind::Colon)
            );

        loop {
            if positional {
                let field_index = fields.len();
                fields.push(StructLiteralField {
                    name: String::new(),
                    value: self.parse_expr_with_hint(
                        field_type_hints
                            .as_ref()
                            .and_then(|hints| hints.get(field_index).map(|(_, typ)| typ.as_str())),
                    )?,
                });
            } else {
                let field_name = self.expect_ident()?;
                fields.push(StructLiteralField {
                    name: field_name.clone(),
                    value: {
                        self.expect_punctuation("`:`", |kind| matches!(kind, TokenKind::Colon))?;
                        self.parse_expr_with_hint(field_type_hints.as_ref().and_then(|hints| {
                            hints
                                .iter()
                                .find(|(name, _)| name == &field_name)
                                .map(|(_, typ)| typ.as_str())
                        }))?
                    },
                });
            }
            if !self.check(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
            self.bump();
            if self.check(|kind| matches!(kind, TokenKind::RBrace)) {
                break;
            }
        }

        self.expect_punctuation("`}`", |kind| matches!(kind, TokenKind::RBrace))?;
        Ok(Expr::StructLiteral { type_name, fields })
    }

    fn looks_like_struct_literal(&self) -> bool {
        if !self.allow_named_struct_literal {
            return false;
        }
        if !self.check(|kind| matches!(kind, TokenKind::LBrace)) {
            return false;
        }

        match self.tokens.get(self.cursor + 1).map(|token| &token.kind) {
            Some(TokenKind::RBrace) => self.allow_empty_struct_literal,
            Some(TokenKind::Ident(_)) => {
                let next = self.tokens.get(self.cursor + 2).map(|token| &token.kind);
                matches!(
                    next,
                    Some(TokenKind::Colon | TokenKind::Comma | TokenKind::RBrace)
                )
            }
            Some(TokenKind::Int(_) | TokenKind::String(_) | TokenKind::True | TokenKind::False) => {
                true
            }
            _ => false,
        }
    }

    fn parse_collection_literal(&mut self) -> Result<Expr, ParseError> {
        self.expect_punctuation("`[`", |kind| matches!(kind, TokenKind::LBracket))?;
        if self.check(|kind| matches!(kind, TokenKind::RBracket)) {
            self.bump();
            let element_type = self.parse_type_name()?;
            if self.check(|kind| matches!(kind, TokenKind::LParen)) {
                self.bump();
                let expr = self.parse_expr_with_hint(None)?;
                self.expect_punctuation("`)`", |kind| matches!(kind, TokenKind::RParen))?;
                return Ok(Expr::SliceConversion {
                    element_type,
                    expr: Box::new(expr),
                });
            }
            let elements = self.parse_collection_elements(&element_type)?;
            return Ok(Expr::SliceLiteral {
                element_type,
                elements,
            });
        }

        let len = match &self.current_token()?.kind {
            TokenKind::Int(value) => {
                let value = *value;
                self.bump();
                value as usize
            }
            other => {
                return Err(ParseError::UnexpectedToken {
                    expected: "array length".into(),
                    found: describe_token(other),
                });
            }
        };
        self.expect_punctuation("`]`", |kind| matches!(kind, TokenKind::RBracket))?;
        let element_type = self.parse_type_name()?;
        let elements = self.parse_collection_elements(&element_type)?;
        Ok(Expr::ArrayLiteral {
            len,
            element_type,
            elements,
        })
    }

    fn parse_collection_elements(&mut self, element_type: &str) -> Result<Vec<Expr>, ParseError> {
        self.expect_punctuation("`{`", |kind| matches!(kind, TokenKind::LBrace))?;
        let mut elements = Vec::new();
        if self.check(|kind| matches!(kind, TokenKind::RBrace)) {
            self.bump();
            return Ok(elements);
        }

        loop {
            elements.push(self.parse_expr_with_hint(Some(element_type))?);
            if !self.check(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
            self.bump();
            if self.check(|kind| matches!(kind, TokenKind::RBrace)) {
                break;
            }
        }

        self.expect_punctuation("`}`", |kind| matches!(kind, TokenKind::RBrace))?;
        Ok(elements)
    }

    fn parse_map_literal(&mut self) -> Result<Expr, ParseError> {
        let map_type = self.parse_type_name()?;
        let Some((key_type, value_type)) = split_map_type_name(&map_type) else {
            return Err(ParseError::UnexpectedToken {
                expected: "map type".into(),
                found: map_type,
            });
        };

        self.expect_punctuation("`{`", |kind| matches!(kind, TokenKind::LBrace))?;
        let mut entries = Vec::new();
        if self.check(|kind| matches!(kind, TokenKind::RBrace)) {
            self.bump();
            return Ok(Expr::MapLiteral {
                key_type: key_type.into(),
                value_type: value_type.into(),
                entries,
            });
        }

        loop {
            let key = self.parse_expr_with_hint(Some(key_type))?;
            self.expect_punctuation("`:`", |kind| matches!(kind, TokenKind::Colon))?;
            let value = self.parse_expr_with_hint(Some(value_type))?;
            entries.push(MapLiteralEntry { key, value });
            if !self.check(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
            self.bump();
            if self.check(|kind| matches!(kind, TokenKind::RBrace)) {
                break;
            }
        }

        self.expect_punctuation("`}`", |kind| matches!(kind, TokenKind::RBrace))?;
        Ok(Expr::MapLiteral {
            key_type: key_type.into(),
            value_type: value_type.into(),
            entries,
        })
    }
}

fn struct_field_type_hints(type_name: &str) -> Option<Vec<(String, String)>> {
    let typ = parse_type_repr(type_name).ok()?;
    let TypeRepr::Struct { fields } = typ else {
        return None;
    };
    Some(
        fields
            .into_iter()
            .map(|field| (field.name, field.typ))
            .collect(),
    )
}

fn split_map_type_name(map_type: &str) -> Option<(&str, &str)> {
    if !map_type.starts_with("map[") {
        return None;
    }

    let mut depth = 1usize;
    let start = 4usize;
    for (offset, ch) in map_type[start..].char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    let end = start + offset;
                    return Some((&map_type[start..end], &map_type[end + 1..]));
                }
            }
            _ => {}
        }
    }

    None
}
