use super::*;

impl Parser {
    pub(super) fn parse_type_decl(&mut self) -> Result<TypeDecl, ParseError> {
        self.expect_keyword("`type`", TokenKind::Type)?;
        let name = self.expect_ident()?;
        let type_params = self.parse_optional_type_params()?;
        if self.check(|kind| matches!(kind, TokenKind::Interface)) {
            self.bump();
            let (methods, embeds) = self.parse_interface_type_body()?;
            return Ok(TypeDecl {
                name,
                type_params,
                kind: TypeDeclKind::Interface { methods, embeds },
            });
        }
        if !self.check(|kind| matches!(kind, TokenKind::Struct)) {
            let underlying = self.parse_type_name()?;
            return Ok(TypeDecl {
                name,
                type_params,
                kind: TypeDeclKind::Alias { underlying },
            });
        }
        self.expect_keyword("`struct`", TokenKind::Struct)?;
        self.expect_punctuation("`{`", |kind| matches!(kind, TokenKind::LBrace))?;
        let mut fields = Vec::new();
        while !self.check(|kind| matches!(kind, TokenKind::RBrace)) {
            if self.check(|kind| matches!(kind, TokenKind::Eof)) {
                return Err(ParseError::UnexpectedEof {
                    context: "struct type declaration".into(),
                });
            }

            let (name, typ, embedded) = if self.check(|kind| matches!(kind, TokenKind::Star)) {
                let typ = self.parse_type_name()?;
                let base = typ.strip_prefix('*').unwrap_or(&typ);
                (base.to_string(), typ, true)
            } else {
                let name = self.expect_ident()?;
                if self.check(|kind| {
                    matches!(
                        kind,
                        TokenKind::Semicolon | TokenKind::RBrace | TokenKind::String(_)
                    )
                }) {
                    (name.clone(), name, true)
                } else {
                    (name, self.parse_type_name()?, false)
                }
            };
            let tag = if self.check(|kind| matches!(kind, TokenKind::String(_))) {
                Some(self.expect_string()?)
            } else {
                None
            };
            fields.push(TypeFieldDecl {
                name,
                typ,
                embedded,
                tag,
            });
            while self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                self.bump();
            }
        }
        self.expect_punctuation("`}`", |kind| matches!(kind, TokenKind::RBrace))?;
        Ok(TypeDecl {
            name,
            type_params,
            kind: TypeDeclKind::Struct { fields },
        })
    }

    fn parse_interface_type_body(
        &mut self,
    ) -> Result<(Vec<InterfaceMethodDecl>, Vec<String>), ParseError> {
        self.expect_punctuation("`{`", |kind| matches!(kind, TokenKind::LBrace))?;
        let mut methods = Vec::new();
        let mut embeds = Vec::new();
        while !self.check(|kind| matches!(kind, TokenKind::RBrace)) {
            if self.check(|kind| matches!(kind, TokenKind::Eof)) {
                return Err(ParseError::UnexpectedEof {
                    context: "interface type declaration".into(),
                });
            }
            let name = self.expect_ident()?;
            if self.check(|kind| matches!(kind, TokenKind::LParen)) {
                self.expect_punctuation("`(`", |kind| matches!(kind, TokenKind::LParen))?;
                let params = self.parse_parameter_list(true)?;
                self.expect_punctuation("`)`", |kind| matches!(kind, TokenKind::RParen))?;
                let result_types = self.parse_result_types()?;
                methods.push(InterfaceMethodDecl {
                    name,
                    params,
                    result_types,
                });
            } else {
                embeds.push(name);
            }
            while self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                self.bump();
            }
        }
        self.expect_punctuation("`}`", |kind| matches!(kind, TokenKind::RBrace))?;
        Ok((methods, embeds))
    }
}
