use super::*;

impl Parser {
    pub(super) fn parse_method_receiver(&mut self) -> Result<Parameter, ParseError> {
        self.expect_punctuation("`(`", |kind| matches!(kind, TokenKind::LParen))?;
        let receiver = if self.check(|kind| matches!(kind, TokenKind::Star)) {
            Parameter {
                name: String::new(),
                typ: self.parse_type_name()?,
                variadic: false,
            }
        } else {
            let first = self.expect_ident()?;
            if self.check(|kind| matches!(kind, TokenKind::RParen)) {
                Parameter {
                    name: String::new(),
                    typ: first,
                    variadic: false,
                }
            } else {
                Parameter {
                    name: first,
                    typ: self.parse_type_name()?,
                    variadic: false,
                }
            }
        };
        self.expect_punctuation("`)`", |kind| matches!(kind, TokenKind::RParen))?;
        Ok(receiver)
    }

    pub(super) fn parse_optional_type_params(&mut self) -> Result<Vec<TypeParam>, ParseError> {
        if !self.check(|kind| matches!(kind, TokenKind::LBracket)) {
            return Ok(Vec::new());
        }
        if !self.peek_is_type_param_list() {
            return Ok(Vec::new());
        }
        self.bump();
        let mut type_params = Vec::new();
        loop {
            let name = self.expect_ident()?;
            let constraint =
                if self.check(|kind| matches!(kind, TokenKind::Comma | TokenKind::RBracket)) {
                    TypeConstraintRepr::Any.render()
                } else {
                    self.parse_type_constraint_repr()?.render()
                };
            type_params.push(TypeParam { name, constraint });
            if !self.check(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
            self.bump();
        }
        self.expect_punctuation("`]`", |kind| matches!(kind, TokenKind::RBracket))?;
        Ok(type_params)
    }

    fn peek_is_type_param_list(&self) -> bool {
        let mut i = self.cursor + 1;
        if !matches!(
            self.tokens.get(i).map(|t| &t.kind),
            Some(TokenKind::Ident(_))
        ) {
            return false;
        }
        i += 1;
        matches!(
            self.tokens.get(i).map(|t| &t.kind),
            Some(
                TokenKind::Ident(_)
                    | TokenKind::Comma
                    | TokenKind::RBracket
                    | TokenKind::Interface
                    | TokenKind::Star
                    | TokenKind::LBracket
                    | TokenKind::Map
                    | TokenKind::Chan
                    | TokenKind::Arrow
            )
        )
    }

    pub(super) fn parse_parameter_list(
        &mut self,
        allow_unnamed: bool,
    ) -> Result<Vec<Parameter>, ParseError> {
        let mut params = Vec::new();
        if self.check(|kind| matches!(kind, TokenKind::RParen)) {
            return Ok(params);
        }

        loop {
            if allow_unnamed && self.peek_parameter_is_type_only() {
                let variadic = self.check(|kind| matches!(kind, TokenKind::Ellipsis));
                if variadic {
                    self.bump();
                }
                let typ = self.parse_type_name()?;
                params.push(Parameter {
                    name: String::new(),
                    typ: if variadic { format!("[]{typ}") } else { typ },
                    variadic,
                });
            } else {
                let mut names = vec![self.expect_ident()?];
                while self.check(|kind| matches!(kind, TokenKind::Comma)) {
                    self.bump();
                    names.push(self.expect_ident()?);
                }
                let variadic = self.check(|kind| matches!(kind, TokenKind::Ellipsis));
                if variadic {
                    self.bump();
                }
                let typ = self.parse_type_name()?;
                let typ = if variadic { format!("[]{typ}") } else { typ };
                for name in names {
                    params.push(Parameter {
                        name,
                        typ: typ.clone(),
                        variadic,
                    });
                }
            }
            if !self.check(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
            self.bump();
            if self.check(|kind| matches!(kind, TokenKind::RParen)) {
                break;
            }
        }

        Ok(params)
    }

    fn peek_parameter_is_type_only(&self) -> bool {
        let mut lookahead = Parser {
            tokens: self.tokens.clone(),
            cursor: self.cursor,
            allow_empty_struct_literal: self.allow_empty_struct_literal,
            allow_named_struct_literal: self.allow_named_struct_literal,
            source_file_spans: SourceFileSpans::empty(),
            stmt_spans: Vec::new(),
        };
        if lookahead.check(|kind| matches!(kind, TokenKind::Ellipsis)) {
            lookahead.bump();
        }
        lookahead.parse_type_repr().is_ok()
            && lookahead.check(|kind| matches!(kind, TokenKind::Comma | TokenKind::RParen))
    }

    pub(super) fn parse_result_types(&mut self) -> Result<Vec<String>, ParseError> {
        if self.starts_type_name() {
            return Ok(vec![self.parse_type_name()?]);
        }

        if !self.check(|kind| matches!(kind, TokenKind::LParen)) {
            return Ok(Vec::new());
        }

        self.expect_punctuation("`(`", |kind| matches!(kind, TokenKind::LParen))?;
        let mut result_types = Vec::new();
        if self.check(|kind| matches!(kind, TokenKind::RParen)) {
            self.expect_punctuation("`)`", |kind| matches!(kind, TokenKind::RParen))?;
            return Ok(result_types);
        }

        loop {
            result_types.push(self.parse_type_name()?);
            if !self.check(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
            self.bump();
            if self.check(|kind| matches!(kind, TokenKind::RParen)) {
                break;
            }
        }

        self.expect_punctuation("`)`", |kind| matches!(kind, TokenKind::RParen))?;
        Ok(result_types)
    }

    fn peek_named_results(&self) -> bool {
        let mut i = self.cursor;
        if !matches!(self.tokens.get(i).map(|t| &t.kind), Some(TokenKind::LParen)) {
            return false;
        }
        i += 1;
        if !matches!(
            self.tokens.get(i).map(|t| &t.kind),
            Some(TokenKind::Ident(_))
        ) {
            return false;
        }
        i += 1;
        matches!(
            self.tokens.get(i).map(|t| &t.kind),
            Some(
                TokenKind::Ident(_)
                    | TokenKind::Star
                    | TokenKind::LBracket
                    | TokenKind::Map
                    | TokenKind::Chan
                    | TokenKind::Arrow
                    | TokenKind::Func
            )
        )
    }

    pub(super) fn parse_result_list(&mut self) -> Result<(Vec<String>, Vec<String>), ParseError> {
        if !self.peek_named_results() {
            let types = self.parse_result_types()?;
            let names = vec![String::new(); types.len()];
            return Ok((names, types));
        }

        self.expect_punctuation("`(`", |kind| matches!(kind, TokenKind::LParen))?;
        let mut names = Vec::new();
        let mut types = Vec::new();

        loop {
            let name = self.expect_ident()?;
            let typ = self.parse_type_name()?;
            names.push(name);
            types.push(typ);
            if !self.check(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
            self.bump();
            if self.check(|kind| matches!(kind, TokenKind::RParen)) {
                break;
            }
        }

        self.expect_punctuation("`)`", |kind| matches!(kind, TokenKind::RParen))?;
        Ok((names, types))
    }
}
