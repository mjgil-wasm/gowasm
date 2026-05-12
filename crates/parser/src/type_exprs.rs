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
            let (methods, embeds) = self.parse_interface_type_body()?;
            if methods.is_empty() && embeds.is_empty() {
                return Ok(TypeRepr::Interface);
            }
            return Ok(TypeRepr::Name(render_interface_literal(&methods, &embeds)));
        }

        if self.check(|kind| matches!(kind, TokenKind::Struct)) {
            self.bump();
            let fields = self.parse_struct_type_fields("anonymous struct type")?;
            return Ok(TypeRepr::Struct { fields });
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
        let params = self
            .parse_parameter_list(true)?
            .into_iter()
            .map(|param| {
                if param.variadic {
                    TypeRepr::Slice(Box::new(
                        parse_type_repr(&param.typ[2..]).expect("canonical variadic type"),
                    ))
                } else {
                    parse_type_repr(&param.typ).expect("canonical parameter type")
                }
            })
            .collect();
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

fn render_interface_literal(methods: &[InterfaceMethodDecl], embeds: &[String]) -> String {
    let mut members = Vec::new();
    members.extend(methods.iter().map(render_interface_method_decl));
    members.extend(embeds.iter().cloned());
    format!("interface{{{}}}", members.join(";"))
}

fn render_interface_method_decl(method: &InterfaceMethodDecl) -> String {
    let params = method
        .params
        .iter()
        .map(|param| {
            let prefix = if param.variadic { "..." } else { "" };
            format!("{} {}{}", param.name, prefix, param.typ)
        })
        .collect::<Vec<_>>()
        .join(",");
    match method.result_types.as_slice() {
        [] => format!("{}({params})", method.name),
        [result] => format!("{}({params}) {result}", method.name),
        results => format!("{}({params}) ({})", method.name, results.join(",")),
    }
}
