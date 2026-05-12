use super::*;

impl Parser {
    pub(super) fn starts_type_name(&self) -> bool {
        self.check(|kind| {
            matches!(
                kind,
                TokenKind::Ident(_)
                    | TokenKind::LBracket
                    | TokenKind::Map
                    | TokenKind::Chan
                    | TokenKind::Arrow
                    | TokenKind::Star
                    | TokenKind::Func
                    | TokenKind::Interface
            )
        })
    }

    pub(super) fn parse_type_name(&mut self) -> Result<String, ParseError> {
        self.parse_type_repr().map(|typ| typ.render())
    }

    pub(super) fn parse_type_repr(&mut self) -> Result<TypeRepr, ParseError> {
        if self.check(|kind| matches!(kind, TokenKind::Func)) {
            return self.parse_function_type_repr();
        }

        if self.check(|kind| matches!(kind, TokenKind::Interface)) {
            self.bump();
            self.expect_punctuation("`{`", |kind| matches!(kind, TokenKind::LBrace))?;
            self.expect_punctuation("`}`", |kind| matches!(kind, TokenKind::RBrace))?;
            return Ok(TypeRepr::Interface);
        }

        if self.check(|kind| matches!(kind, TokenKind::Struct)) {
            self.bump();
            let fields = self.parse_struct_fields("anonymous struct type")?;
            return Ok(TypeRepr::Name(render_struct_type_name(&fields)));
        }

        if self.check(|kind| matches!(kind, TokenKind::Star)) {
            self.bump();
            return Ok(TypeRepr::Pointer(Box::new(self.parse_type_repr()?)));
        }

        if self.check(|kind| matches!(kind, TokenKind::Map)) {
            self.bump();
            self.expect_punctuation("`[`", |kind| matches!(kind, TokenKind::LBracket))?;
            let key = self.parse_type_repr()?;
            self.expect_punctuation("`]`", |kind| matches!(kind, TokenKind::RBracket))?;
            let value = self.parse_type_repr()?;
            return Ok(TypeRepr::Map {
                key: Box::new(key),
                value: Box::new(value),
            });
        }

        if self.check(|kind| matches!(kind, TokenKind::Arrow)) {
            self.bump();
            self.expect_keyword("`chan`", TokenKind::Chan)?;
            return Ok(TypeRepr::Channel {
                direction: TypeChannelDirection::ReceiveOnly,
                element: Box::new(self.parse_type_repr()?),
            });
        }

        if self.check(|kind| matches!(kind, TokenKind::Chan)) {
            self.bump();
            let direction = if self.check(|kind| matches!(kind, TokenKind::Arrow)) {
                self.bump();
                TypeChannelDirection::SendOnly
            } else {
                TypeChannelDirection::Bidirectional
            };
            return Ok(TypeRepr::Channel {
                direction,
                element: Box::new(self.parse_type_repr()?),
            });
        }

        if self.check(|kind| matches!(kind, TokenKind::Ident(_))) {
            let mut name = self.expect_ident()?;
            if self.check(|kind| matches!(kind, TokenKind::Dot)) {
                self.bump();
                name.push('.');
                name.push_str(&self.expect_ident()?);
            }
            if name == "any" {
                return Ok(TypeRepr::Interface);
            }
            if self.check(|kind| matches!(kind, TokenKind::LBracket)) {
                self.bump();
                let type_args = self.parse_type_repr_list(TokenKind::RBracket)?;
                self.expect_punctuation("`]`", |kind| matches!(kind, TokenKind::RBracket))?;
                return Ok(TypeRepr::GenericInstance {
                    base: name,
                    type_args,
                });
            }
            return Ok(TypeRepr::Name(name));
        }

        self.expect_punctuation("`[`", |kind| matches!(kind, TokenKind::LBracket))?;
        if self.check(|kind| matches!(kind, TokenKind::RBracket)) {
            self.bump();
            return Ok(TypeRepr::Slice(Box::new(self.parse_type_repr()?)));
        }

        let len = match &self.current_token()?.kind {
            TokenKind::Int(value) => {
                let value = usize::try_from(*value).map_err(|_| ParseError::UnexpectedToken {
                    expected: "non-negative array length".into(),
                    found: value.to_string(),
                })?;
                self.bump();
                value
            }
            other => {
                return Err(ParseError::UnexpectedToken {
                    expected: "array length".into(),
                    found: describe_token(other),
                });
            }
        };
        self.expect_punctuation("`]`", |kind| matches!(kind, TokenKind::RBracket))?;
        Ok(TypeRepr::Array {
            len,
            element: Box::new(self.parse_type_repr()?),
        })
    }

    fn parse_function_type_repr(&mut self) -> Result<TypeRepr, ParseError> {
        self.expect_keyword("`func`", TokenKind::Func)?;
        self.expect_punctuation("`(`", |kind| matches!(kind, TokenKind::LParen))?;
        let params = self.parse_type_repr_list(TokenKind::RParen)?;
        self.expect_punctuation("`)`", |kind| matches!(kind, TokenKind::RParen))?;
        let results = self.parse_function_type_results_repr()?;
        Ok(TypeRepr::Function { params, results })
    }

    fn parse_function_type_results_repr(&mut self) -> Result<Vec<TypeRepr>, ParseError> {
        if self.starts_type_name() {
            return Ok(vec![self.parse_type_repr()?]);
        }

        if !self.check(|kind| matches!(kind, TokenKind::LParen)) {
            return Ok(Vec::new());
        }

        self.expect_punctuation("`(`", |kind| matches!(kind, TokenKind::LParen))?;
        let results = self.parse_type_repr_list(TokenKind::RParen)?;
        self.expect_punctuation("`)`", |kind| matches!(kind, TokenKind::RParen))?;
        Ok(results)
    }

    fn parse_type_repr_list(&mut self, terminator: TokenKind) -> Result<Vec<TypeRepr>, ParseError> {
        let mut types = Vec::new();
        if self.check(|kind| kind == &terminator) {
            return Ok(types);
        }

        loop {
            types.push(self.parse_type_repr()?);
            if !self.check(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
            self.bump();
            if self.check(|kind| kind == &terminator) {
                break;
            }
        }

        Ok(types)
    }
}

fn render_struct_type_name(fields: &[TypeFieldDecl]) -> String {
    if fields.is_empty() {
        return "struct{}".into();
    }

    let fields = fields
        .iter()
        .map(|field| {
            let mut rendered = if field.embedded {
                field.typ.clone()
            } else {
                format!("{} {}", field.name, field.typ)
            };
            if let Some(tag) = &field.tag {
                rendered.push(' ');
                rendered.push('`');
                rendered.push_str(tag);
                rendered.push('`');
            }
            rendered
        })
        .collect::<Vec<_>>()
        .join(";");
    format!("struct{{{fields}}}")
}
