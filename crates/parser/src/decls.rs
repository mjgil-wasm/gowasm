use super::*;

impl Parser {
    pub(super) fn parse_var_decl(&mut self) -> Result<Stmt, ParseError> {
        self.expect_keyword("`var`", TokenKind::Var)?;
        let (name, typ, value) = self.parse_var_binding()?;
        Ok(Stmt::VarDecl { name, typ, value })
    }

    pub(super) fn parse_const_decl(&mut self) -> Result<Stmt, ParseError> {
        self.expect_keyword("`const`", TokenKind::Const)?;
        if self.check(|kind| matches!(kind, TokenKind::LParen)) {
            return Ok(Stmt::ConstGroup {
                decls: self.parse_const_group_decls()?,
            });
        }
        let (name, typ, value) = self.parse_var_binding()?;
        let value = value.ok_or_else(|| ParseError::UnexpectedToken {
            expected: "const initializer".into(),
            found: describe_token(
                self.current_token()
                    .map(|token| &token.kind)
                    .unwrap_or(&TokenKind::Eof),
            ),
        })?;
        Ok(Stmt::ConstDecl {
            name,
            typ,
            value,
            iota: 0,
        })
    }

    pub(super) fn parse_package_var_decl(&mut self) -> Result<PackageVarDecl, ParseError> {
        let start = self.current_token()?.span.start;
        self.expect_keyword("`var`", TokenKind::Var)?;
        let (name, typ, value) = self.parse_var_binding()?;
        self.source_file_spans.vars.push(Span {
            start,
            end: self.last_consumed_end(start),
        });
        Ok(PackageVarDecl { name, typ, value })
    }

    pub(super) fn parse_package_const_decls(
        &mut self,
    ) -> Result<Vec<PackageConstDecl>, ParseError> {
        let start = self.current_token()?.span.start;
        self.expect_keyword("`const`", TokenKind::Const)?;
        if self.check(|kind| matches!(kind, TokenKind::LParen)) {
            let decls = self.parse_package_const_group()?;
            let span = Span {
                start,
                end: self.last_consumed_end(start),
            };
            self.source_file_spans
                .consts
                .extend(std::iter::repeat_n(span, decls.len()));
            return Ok(decls);
        }
        let (name, typ, value) = self.parse_var_binding()?;
        let value = value.ok_or_else(|| ParseError::UnexpectedToken {
            expected: "const initializer".into(),
            found: describe_token(
                self.current_token()
                    .map(|token| &token.kind)
                    .unwrap_or(&TokenKind::Eof),
            ),
        })?;
        self.source_file_spans.consts.push(Span {
            start,
            end: self.last_consumed_end(start),
        });
        Ok(vec![PackageConstDecl {
            name,
            typ,
            value,
            iota: 0,
        }])
    }

    fn parse_const_group_decls(&mut self) -> Result<Vec<ConstGroupDecl>, ParseError> {
        let bindings = self.parse_const_group_bindings()?;
        Ok(bindings
            .into_iter()
            .enumerate()
            .map(|(iota, (name, typ, value))| ConstGroupDecl {
                name,
                typ,
                value,
                iota,
            })
            .collect())
    }

    fn parse_package_const_group(&mut self) -> Result<Vec<PackageConstDecl>, ParseError> {
        let bindings = self.parse_const_group_bindings()?;
        Ok(bindings
            .into_iter()
            .enumerate()
            .map(|(iota, (name, typ, value))| PackageConstDecl {
                name,
                typ,
                value,
                iota,
            })
            .collect())
    }

    fn parse_const_group_bindings(
        &mut self,
    ) -> Result<Vec<(String, Option<String>, Expr)>, ParseError> {
        self.expect_punctuation("`(`", |kind| matches!(kind, TokenKind::LParen))?;
        let mut decls = Vec::new();
        let mut previous_spec = None;
        while !self.check(|kind| matches!(kind, TokenKind::RParen)) {
            while self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                self.bump();
            }
            if self.check(|kind| matches!(kind, TokenKind::RParen)) {
                break;
            }
            let (name, mut typ, value) = self.parse_const_group_binding()?;
            let value = match value {
                Some(value) => {
                    previous_spec = Some((typ.clone(), value.clone()));
                    value
                }
                None => {
                    let (previous_typ, previous_value) =
                        previous_spec
                            .clone()
                            .ok_or_else(|| ParseError::UnexpectedToken {
                                expected: "const initializer".into(),
                                found: describe_token(
                                    self.current_token()
                                        .map(|token| &token.kind)
                                        .unwrap_or(&TokenKind::Eof),
                                ),
                            })?;
                    if typ.is_none() {
                        typ = previous_typ;
                    }
                    previous_value
                }
            };
            decls.push((name, typ, value));
            while self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                self.bump();
            }
        }
        self.expect_punctuation("`)`", |kind| matches!(kind, TokenKind::RParen))?;
        Ok(decls)
    }

    fn parse_const_group_binding(
        &mut self,
    ) -> Result<(String, Option<String>, Option<Expr>), ParseError> {
        let name = self.expect_ident()?;
        let typ = if self.starts_type_name() {
            Some(self.parse_type_name()?)
        } else {
            None
        };

        let value = if self.check(|kind| matches!(kind, TokenKind::Equal)) {
            self.bump();
            Some(self.parse_expr()?)
        } else {
            None
        };

        Ok((name, typ, value))
    }

    fn parse_var_binding(&mut self) -> Result<(String, Option<String>, Option<Expr>), ParseError> {
        let name = self.expect_ident()?;
        let typ = if self.starts_type_name() {
            Some(self.parse_type_name()?)
        } else {
            None
        };

        let value = if self.check(|kind| matches!(kind, TokenKind::Equal)) {
            self.bump();
            Some(self.parse_expr()?)
        } else {
            None
        };

        if typ.is_none() && value.is_none() {
            return Err(ParseError::UnexpectedToken {
                expected: "type name or `=`".into(),
                found: describe_token(&self.current_token()?.kind),
            });
        }

        Ok((name, typ, value))
    }
}
