use super::*;

impl Parser {
    pub(super) fn parse_type_constraint_repr(&mut self) -> Result<TypeConstraintRepr, ParseError> {
        if self.check(|kind| matches!(kind, TokenKind::Interface)) {
            self.bump();
            let interface = self.parse_constraint_interface_body()?;
            if interface.methods.is_empty()
                && interface.embeds.is_empty()
                && interface.type_sets.is_empty()
            {
                return Ok(TypeConstraintRepr::Any);
            }
            return Ok(TypeConstraintRepr::Interface(interface));
        }

        let constraint = self.parse_type_name()?;
        Ok(match constraint.as_str() {
            "interface{}" => TypeConstraintRepr::Any,
            "comparable" => TypeConstraintRepr::Comparable,
            name => TypeConstraintRepr::Named(name.to_string()),
        })
    }

    fn parse_constraint_interface_body(
        &mut self,
    ) -> Result<TypeConstraintInterfaceRepr, ParseError> {
        self.expect_punctuation("`{`", |kind| matches!(kind, TokenKind::LBrace))?;
        let mut methods = Vec::new();
        let mut embeds = Vec::new();
        let mut type_sets = Vec::new();

        while !self.check(|kind| matches!(kind, TokenKind::RBrace)) {
            if self.check(|kind| matches!(kind, TokenKind::Eof)) {
                return Err(ParseError::UnexpectedEof {
                    context: "type constraint".into(),
                });
            }

            let first = self.parse_type_name()?;
            if self.check(|kind| matches!(kind, TokenKind::LParen)) && is_simple_name(&first) {
                self.expect_punctuation("`(`", |kind| matches!(kind, TokenKind::LParen))?;
                let params = self.parse_parameter_list(true)?;
                self.expect_punctuation("`)`", |kind| matches!(kind, TokenKind::RParen))?;
                let result_types = self.parse_result_types()?;
                methods.push(InterfaceMethodDecl {
                    name: first,
                    params,
                    result_types,
                });
            } else if self.check(|kind| matches!(kind, TokenKind::BitOr)) {
                let mut terms = vec![first];
                while self.check(|kind| matches!(kind, TokenKind::BitOr)) {
                    self.bump();
                    terms.push(self.parse_type_name()?);
                }
                type_sets.push(terms);
            } else {
                embeds.push(first);
            }

            while self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                self.bump();
            }
        }

        self.expect_punctuation("`}`", |kind| matches!(kind, TokenKind::RBrace))?;
        Ok(TypeConstraintInterfaceRepr {
            methods,
            embeds,
            type_sets,
        })
    }
}

fn is_simple_name(name: &str) -> bool {
    name.as_bytes()
        .iter()
        .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
}
