mod ast;
mod constraints;
mod decls;
mod expr;
mod imports;
mod lookahead;
mod signatures;
mod simple_stmts;
mod spans;
mod token_display;
mod type_decls;
mod type_exprs;

pub use ast::*;
pub use spans::*;

use gowasm_lexer::{lex, Span, Token, TokenKind};
use token_display::describe_token;

pub fn parse_type_repr(source: &str) -> Result<TypeRepr, ParseError> {
    let tokens = lex(source)?;
    let mut parser = Parser {
        tokens,
        cursor: 0,
        allow_empty_struct_literal: true,
        allow_named_struct_literal: true,
        source_file_spans: SourceFileSpans::empty(),
        stmt_spans: Vec::new(),
    };
    let typ = parser.parse_type_repr()?;
    parser.expect_eof()?;
    Ok(typ)
}

pub fn parse_type_constraint(source: &str) -> Result<TypeConstraintRepr, ParseError> {
    let tokens = lex(source)?;
    let mut parser = Parser {
        tokens,
        cursor: 0,
        allow_empty_struct_literal: true,
        allow_named_struct_literal: true,
        source_file_spans: SourceFileSpans::empty(),
        stmt_spans: Vec::new(),
    };
    let constraint = parser.parse_type_constraint_repr()?;
    parser.expect_eof()?;
    Ok(constraint)
}

pub fn parse_source_file(source: &str) -> Result<SourceFile, ParseError> {
    parse_source_file_with_spans(source).map(|(file, _spans)| file)
}

pub fn parse_source_file_with_spans(
    source: &str,
) -> Result<(SourceFile, SourceFileSpans), ParseError> {
    let tokens = lex(source)?;
    let mut parser = Parser {
        tokens,
        cursor: 0,
        allow_empty_struct_literal: true,
        allow_named_struct_literal: true,
        source_file_spans: SourceFileSpans::empty(),
        stmt_spans: Vec::new(),
    };
    let file = parser.parse_source_file()?;
    Ok((file, parser.source_file_spans))
}

struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
    allow_empty_struct_literal: bool,
    allow_named_struct_literal: bool,
    source_file_spans: SourceFileSpans,
    stmt_spans: Vec<Span>,
}

impl Parser {
    fn parse_source_file(&mut self) -> Result<SourceFile, ParseError> {
        self.expect_keyword("package", TokenKind::Package)?;
        let package_name = self.expect_ident()?;
        while self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
            self.bump();
        }

        let mut imports = Vec::new();
        while self.check(|kind| matches!(kind, TokenKind::Import)) {
            imports.extend(self.parse_import_decls()?);
            while self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                self.bump();
            }
        }

        let mut types = Vec::new();
        let mut consts = Vec::new();
        let mut vars = Vec::new();
        let mut functions = Vec::new();
        while !self.check(|kind| matches!(kind, TokenKind::Eof)) {
            if self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                self.bump();
                continue;
            }
            if self.check(|kind| matches!(kind, TokenKind::Type)) {
                types.extend(self.parse_type_decls()?);
                continue;
            }

            if self.check(|kind| matches!(kind, TokenKind::Const)) {
                consts.extend(self.parse_package_const_decls()?);
                continue;
            }

            if self.check(|kind| matches!(kind, TokenKind::Var)) {
                vars.push(self.parse_package_var_decl()?);
                continue;
            }

            if self.check(|kind| matches!(kind, TokenKind::Func)) {
                functions.push(self.parse_function_decl()?);
                continue;
            }

            return Err(ParseError::UnexpectedToken {
                expected: "`type`, `const`, `var`, `func`, or end of file".into(),
                found: describe_token(&self.current_token()?.kind),
            });
        }

        self.expect_eof()?;
        Ok(SourceFile {
            package_name,
            imports,
            types,
            consts,
            vars,
            functions,
        })
    }

    fn parse_function_decl(&mut self) -> Result<FunctionDecl, ParseError> {
        let start = self.current_token()?.span.start;
        let stmt_start = self.stmt_spans.len();
        self.expect_keyword("func", TokenKind::Func)?;
        let receiver = if self.check(|kind| matches!(kind, TokenKind::LParen)) {
            Some(self.parse_method_receiver()?)
        } else {
            None
        };
        let name = self.expect_ident()?;
        let type_params = self.parse_optional_type_params()?;
        self.expect_punctuation("`(`", |kind| matches!(kind, TokenKind::LParen))?;
        let params = self.parse_parameter_list(false)?;
        self.expect_punctuation("`)`", |kind| matches!(kind, TokenKind::RParen))?;
        let (result_names, result_types) = self.parse_result_list()?;
        let body = self.parse_block()?;
        let function = FunctionDecl {
            receiver,
            name,
            type_params,
            params,
            result_types,
            result_names,
            body,
        };
        self.source_file_spans.functions.push(FunctionSourceSpans {
            span: Span {
                start,
                end: self.last_consumed_end(start),
            },
            stmt_spans: self.stmt_spans[stmt_start..].to_vec(),
        });
        Ok(function)
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.expect_punctuation("`{`", |kind| matches!(kind, TokenKind::LBrace))?;
        let mut body = Vec::new();
        while !self.check(|kind| matches!(kind, TokenKind::RBrace)) {
            if self.check(|kind| matches!(kind, TokenKind::Eof)) {
                return Err(ParseError::UnexpectedEof {
                    context: "block".into(),
                });
            }
            body.push(self.parse_stmt()?);
            while self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                self.bump();
            }
        }
        self.expect_punctuation("`}`", |kind| matches!(kind, TokenKind::RBrace))?;
        Ok(body)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        let start = self.current_token()?.span.start;
        let stmt = if self.check(|kind| matches!(kind, TokenKind::Var)) {
            self.parse_var_decl()?
        } else if self.check(|kind| matches!(kind, TokenKind::Const)) {
            self.parse_const_decl()?
        } else if self.check(|kind| matches!(kind, TokenKind::Switch)) {
            self.parse_switch_stmt()?
        } else if self.check(|kind| matches!(kind, TokenKind::Select)) {
            self.parse_select_stmt()?
        } else if self.check(|kind| matches!(kind, TokenKind::For)) {
            self.parse_for_stmt()?
        } else if self.check(|kind| matches!(kind, TokenKind::If)) {
            self.parse_if_stmt()?
        } else if self.check(|kind| matches!(kind, TokenKind::Go)) {
            self.bump();
            let call = self.parse_callable_expr()?;
            Stmt::Go { call }
        } else if self.check(|kind| matches!(kind, TokenKind::Defer)) {
            self.bump();
            let call = self.parse_callable_expr()?;
            Stmt::Defer { call }
        } else if self.check(|kind| matches!(kind, TokenKind::Break)) {
            self.bump();
            let label = self.try_consume_label();
            Stmt::Break { label }
        } else if self.check(|kind| matches!(kind, TokenKind::Continue)) {
            self.bump();
            let label = self.try_consume_label();
            Stmt::Continue { label }
        } else if let Some(label) = self.peek_labeled_stmt()? {
            self.bump();
            self.bump();
            let stmt = self.parse_stmt()?;
            Stmt::Labeled {
                label,
                stmt: Box::new(stmt),
            }
        } else if self.check(|kind| matches!(kind, TokenKind::Return)) {
            self.bump();
            if self.check(|kind| {
                matches!(
                    kind,
                    TokenKind::RBrace | TokenKind::Semicolon | TokenKind::Case | TokenKind::Default
                )
            }) {
                Stmt::Return(Vec::new())
            } else {
                Stmt::Return(self.parse_return_exprs()?)
            }
        } else {
            self.parse_simple_stmt()?
        };
        self.stmt_spans.push(Span {
            start,
            end: self.last_consumed_end(start),
        });
        Ok(stmt)
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.expect_keyword("`if`", TokenKind::If)?;
        let (init, condition) = if self.for_header_has_semicolon() {
            let init_stmt = self.parse_simple_stmt()?;
            self.expect_punctuation("`;`", |kind| matches!(kind, TokenKind::Semicolon))?;
            let condition = self.parse_expr()?;
            (Some(Box::new(init_stmt)), condition)
        } else {
            (None, self.parse_expr()?)
        };
        let then_body = self.parse_block()?;
        let else_body = if self.check(|kind| matches!(kind, TokenKind::Else)) {
            self.bump();
            if self.check(|kind| matches!(kind, TokenKind::If)) {
                let start = self.current_token()?.span.start;
                let else_if = self.parse_if_stmt()?;
                self.stmt_spans.push(Span {
                    start,
                    end: self.last_consumed_end(start),
                });
                Some(vec![else_if])
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };

        Ok(Stmt::If {
            init,
            condition,
            then_body,
            else_body,
        })
    }

    fn parse_for_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.expect_keyword("`for`", TokenKind::For)?;
        if self.check(|kind| matches!(kind, TokenKind::LBrace)) {
            let body = self.parse_block()?;
            return Ok(Stmt::For {
                init: None,
                condition: None,
                post: None,
                body,
            });
        }

        if self.for_header_has_semicolon() {
            let init = if self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                None
            } else {
                Some(Box::new(self.parse_simple_stmt()?))
            };
            self.expect_punctuation("`;`", |kind| matches!(kind, TokenKind::Semicolon))?;
            let condition = if self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                None
            } else {
                Some(self.parse_expr()?)
            };
            self.expect_punctuation("`;`", |kind| matches!(kind, TokenKind::Semicolon))?;
            let post = if self.check(|kind| matches!(kind, TokenKind::LBrace)) {
                None
            } else {
                Some(Box::new(self.parse_simple_stmt()?))
            };
            let body = self.parse_block()?;
            return Ok(Stmt::For {
                init,
                condition,
                post,
                body,
            });
        }

        if self.for_header_has_range() {
            if self.check(|kind| matches!(kind, TokenKind::Range)) {
                self.bump();
                let expr = self.parse_expr_disallowing_named_struct_literal()?;
                let body = self.parse_block()?;
                return Ok(Stmt::RangeFor {
                    key: "_".into(),
                    value: None,
                    assign: false,
                    expr,
                    body,
                });
            }

            let key = self.expect_ident()?;
            let value = if self.check(|kind| matches!(kind, TokenKind::Comma)) {
                self.bump();
                Some(self.expect_ident()?)
            } else {
                None
            };
            let assign = if self.check(|kind| matches!(kind, TokenKind::ColonEqual)) {
                self.bump();
                false
            } else {
                self.expect_punctuation("`=`", |kind| matches!(kind, TokenKind::Equal))?;
                true
            };
            self.expect_keyword("`range`", TokenKind::Range)?;
            let expr = self.parse_expr_disallowing_named_struct_literal()?;
            let body = self.parse_block()?;
            return Ok(Stmt::RangeFor {
                key,
                value,
                assign,
                expr,
                body,
            });
        }

        let condition = Some(self.parse_expr()?);
        let body = self.parse_block()?;
        Ok(Stmt::For {
            init: None,
            condition,
            post: None,
            body,
        })
    }

    fn parse_switch_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.expect_keyword("`switch`", TokenKind::Switch)?;

        let init = if self.for_header_has_semicolon() {
            let init_stmt = self.parse_simple_stmt()?;
            self.expect_punctuation("`;`", |kind| matches!(kind, TokenKind::Semicolon))?;
            Some(Box::new(init_stmt))
        } else {
            None
        };

        if let Some(binding) = self.peek_type_switch() {
            return self.parse_type_switch(init, binding);
        }

        let expr = if self.check(|kind| matches!(kind, TokenKind::LBrace)) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        self.expect_punctuation("`{`", |kind| matches!(kind, TokenKind::LBrace))?;

        let mut cases = Vec::new();
        let mut default = None;
        let mut default_index = None;
        let mut default_fallthrough = false;
        while !self.check(|kind| matches!(kind, TokenKind::RBrace)) {
            if self.check(|kind| matches!(kind, TokenKind::Case)) {
                self.bump();
                cases.push(self.parse_switch_case()?);
                continue;
            }

            if self.check(|kind| matches!(kind, TokenKind::Default)) {
                self.bump();
                if default.is_some() {
                    return Err(ParseError::UnexpectedToken {
                        expected: "at most one `default` clause per `switch`".into(),
                        found: "`default`".into(),
                    });
                }
                self.expect_punctuation("`:`", |kind| matches!(kind, TokenKind::Colon))?;
                default_index = Some(cases.len());
                default = Some(self.parse_switch_clause_body()?);
                if self.check(|kind| matches!(kind, TokenKind::Fallthrough)) {
                    default_fallthrough = true;
                    self.bump();
                    while self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                        self.bump();
                    }
                }
                continue;
            }

            return Err(ParseError::UnexpectedToken {
                expected: "`case`, `default`, or `}`".into(),
                found: describe_token(&self.current_token()?.kind),
            });
        }

        self.expect_punctuation("`}`", |kind| matches!(kind, TokenKind::RBrace))?;
        Ok(Stmt::Switch {
            init,
            expr,
            cases,
            default,
            default_index,
            default_fallthrough,
        })
    }

    fn parse_type_switch(
        &mut self,
        init: Option<Box<Stmt>>,
        binding: Option<String>,
    ) -> Result<Stmt, ParseError> {
        if binding.is_some() {
            self.bump(); // ident
            self.bump(); // :=
        }
        let expr = self.parse_expr()?;
        self.expect_punctuation("`.`", |kind| matches!(kind, TokenKind::Dot))?;
        self.expect_punctuation("`(`", |kind| matches!(kind, TokenKind::LParen))?;
        self.expect_keyword("`type`", TokenKind::Type)?;
        self.expect_punctuation("`)`", |kind| matches!(kind, TokenKind::RParen))?;
        self.expect_punctuation("`{`", |kind| matches!(kind, TokenKind::LBrace))?;

        let mut cases = Vec::new();
        let mut default = None;
        let mut default_index = None;
        while !self.check(|kind| matches!(kind, TokenKind::RBrace)) {
            if self.check(|kind| matches!(kind, TokenKind::Case)) {
                self.bump();
                cases.push(self.parse_type_switch_case()?);
                continue;
            }
            if self.check(|kind| matches!(kind, TokenKind::Default)) {
                self.bump();
                if default.is_some() {
                    return Err(ParseError::UnexpectedToken {
                        expected: "at most one `default` clause per type `switch`".into(),
                        found: "`default`".into(),
                    });
                }
                self.expect_punctuation("`:`", |kind| matches!(kind, TokenKind::Colon))?;
                default_index = Some(cases.len());
                default = Some(self.parse_switch_clause_body()?);
                continue;
            }
            return Err(ParseError::UnexpectedToken {
                expected: "`case`, `default`, or `}`".into(),
                found: describe_token(&self.current_token()?.kind),
            });
        }
        self.expect_punctuation("`}`", |kind| matches!(kind, TokenKind::RBrace))?;

        Ok(Stmt::TypeSwitch {
            init,
            binding,
            expr,
            cases,
            default,
            default_index,
        })
    }

    fn parse_type_switch_case(&mut self) -> Result<TypeSwitchCase, ParseError> {
        let mut types = vec![if self.check(|kind| matches!(kind, TokenKind::Nil)) {
            self.bump();
            "nil".to_string()
        } else {
            self.parse_type_name()?
        }];
        while self.check(|kind| matches!(kind, TokenKind::Comma)) {
            self.bump();
            types.push(if self.check(|kind| matches!(kind, TokenKind::Nil)) {
                self.bump();
                "nil".to_string()
            } else {
                self.parse_type_name()?
            });
        }
        self.expect_punctuation("`:`", |kind| matches!(kind, TokenKind::Colon))?;
        let body = self.parse_switch_clause_body()?;
        Ok(TypeSwitchCase { types, body })
    }

    fn parse_select_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.expect_keyword("`select`", TokenKind::Select)?;
        self.expect_punctuation("`{`", |kind| matches!(kind, TokenKind::LBrace))?;

        let mut cases = Vec::new();
        let mut default = None;
        while !self.check(|kind| matches!(kind, TokenKind::RBrace)) {
            if self.check(|kind| matches!(kind, TokenKind::Case)) {
                self.bump();
                let stmt = self.parse_simple_stmt()?;
                self.expect_punctuation("`:`", |kind| matches!(kind, TokenKind::Colon))?;
                cases.push(SelectCase {
                    stmt,
                    body: self.parse_clause_body("select clause")?,
                });
                continue;
            }

            if self.check(|kind| matches!(kind, TokenKind::Default)) {
                self.bump();
                if default.is_some() {
                    return Err(ParseError::UnexpectedToken {
                        expected: "at most one `default` clause per `select`".into(),
                        found: "`default`".into(),
                    });
                }
                self.expect_punctuation("`:`", |kind| matches!(kind, TokenKind::Colon))?;
                default = Some(self.parse_clause_body("select clause")?);
                continue;
            }

            return Err(ParseError::UnexpectedToken {
                expected: "`case`, `default`, or `}`".into(),
                found: describe_token(&self.current_token()?.kind),
            });
        }

        self.expect_punctuation("`}`", |kind| matches!(kind, TokenKind::RBrace))?;
        Ok(Stmt::Select { cases, default })
    }

    fn parse_switch_case(&mut self) -> Result<SwitchCase, ParseError> {
        let mut expressions = Vec::new();
        loop {
            expressions.push(self.parse_expr()?);
            if !self.check(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
            self.bump();
        }
        self.expect_punctuation("`:`", |kind| matches!(kind, TokenKind::Colon))?;
        let body = self.parse_clause_body("switch clause")?;
        let fallthrough = self.check(|kind| matches!(kind, TokenKind::Fallthrough));
        if fallthrough {
            self.bump();
            while self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                self.bump();
            }
        }
        Ok(SwitchCase {
            expressions,
            body,
            fallthrough,
        })
    }

    fn parse_switch_clause_body(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.parse_clause_body("switch clause")
    }

    fn parse_clause_body(&mut self, context: &str) -> Result<Vec<Stmt>, ParseError> {
        let mut body = Vec::new();
        while !self.check(|kind| {
            matches!(
                kind,
                TokenKind::Case | TokenKind::Default | TokenKind::RBrace | TokenKind::Fallthrough
            )
        }) {
            if self.check(|kind| matches!(kind, TokenKind::Eof)) {
                return Err(ParseError::UnexpectedEof {
                    context: context.into(),
                });
            }
            body.push(self.parse_stmt()?);
            while self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
                self.bump();
            }
        }
        Ok(body)
    }

    fn for_header_has_semicolon(&self) -> bool {
        let mut cursor = self.cursor;
        while let Some(token) = self.tokens.get(cursor) {
            match token.kind {
                TokenKind::Semicolon => return true,
                TokenKind::LBrace | TokenKind::Eof => return false,
                _ => cursor += 1,
            }
        }

        false
    }

    fn for_header_has_range(&self) -> bool {
        let mut cursor = self.cursor;
        while let Some(token) = self.tokens.get(cursor) {
            match token.kind {
                TokenKind::Range => return true,
                TokenKind::LBrace | TokenKind::Eof => return false,
                _ => cursor += 1,
            }
        }

        false
    }

    fn parse_return_exprs(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut values = Vec::new();
        loop {
            values.push(self.parse_expr()?);
            if !self.check(|kind| matches!(kind, TokenKind::Comma)) {
                break;
            }
            self.bump();
        }
        Ok(values)
    }

    fn parse_expr_disallowing_named_struct_literal(&mut self) -> Result<Expr, ParseError> {
        let allow_named_struct_literal = self.allow_named_struct_literal;
        self.allow_named_struct_literal = false;
        let expr = self.parse_expr();
        self.allow_named_struct_literal = allow_named_struct_literal;
        expr
    }

    fn expect_keyword(&mut self, label: &str, expected: TokenKind) -> Result<(), ParseError> {
        let token = self.current_token()?;
        if token.kind == expected {
            self.bump();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: label.into(),
                found: describe_token(&token.kind),
            })
        }
    }

    fn expect_punctuation(
        &mut self,
        label: &str,
        predicate: impl Fn(&TokenKind) -> bool,
    ) -> Result<(), ParseError> {
        let token = self.current_token()?;
        if predicate(&token.kind) {
            self.bump();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: label.into(),
                found: describe_token(&token.kind),
            })
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match &self.current_token()?.kind {
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.bump();
                Ok(name)
            }
            other => Err(ParseError::UnexpectedToken {
                expected: "identifier".into(),
                found: describe_token(other),
            }),
        }
    }

    fn expect_string(&mut self) -> Result<String, ParseError> {
        match &self.current_token()?.kind {
            TokenKind::String(value) => {
                let value = value.clone();
                self.bump();
                Ok(value)
            }
            other => Err(ParseError::UnexpectedToken {
                expected: "string literal".into(),
                found: describe_token(other),
            }),
        }
    }

    fn expect_eof(&mut self) -> Result<(), ParseError> {
        let token = self.current_token()?;
        if matches!(token.kind, TokenKind::Eof) {
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "end of file".into(),
                found: describe_token(&token.kind),
            })
        }
    }

    fn current_token(&self) -> Result<&Token, ParseError> {
        self.tokens
            .get(self.cursor)
            .ok_or(ParseError::UnexpectedEof {
                context: "token stream".into(),
            })
    }

    fn check(&self, predicate: impl Fn(&TokenKind) -> bool) -> bool {
        self.tokens
            .get(self.cursor)
            .map(|token| predicate(&token.kind))
            .unwrap_or(false)
    }

    fn bump(&mut self) {
        self.cursor += 1;
    }

    fn last_consumed_end(&self, fallback: usize) -> usize {
        self.tokens
            .get(self.cursor.saturating_sub(1))
            .map(|token| token.span.end)
            .unwrap_or(fallback)
    }

    fn skip_semicolons(&mut self) {
        while self.check(|kind| matches!(kind, TokenKind::Semicolon)) {
            self.bump();
        }
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_bitwise;
#[cfg(test)]
mod tests_builtins;
#[cfg(test)]
mod tests_concurrency;
#[cfg(test)]
mod tests_const;
#[cfg(test)]
mod tests_control_flow;
#[cfg(test)]
mod tests_functions;
#[cfg(test)]
mod tests_generics;
#[cfg(test)]
mod tests_globals;
#[cfg(test)]
mod tests_imports;
#[cfg(test)]
mod tests_interfaces;
#[cfg(test)]
mod tests_maps;
#[cfg(test)]
mod tests_multi_results;
#[cfg(test)]
mod tests_pointers;
#[cfg(test)]
mod tests_range;
#[cfg(test)]
mod tests_select;
#[cfg(test)]
mod tests_structs;
#[cfg(test)]
mod tests_type_repr;
#[cfg(test)]
mod tests_unwind;
#[cfg(test)]
mod tests_upstream;
