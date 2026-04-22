use super::*;

impl Parser {
    pub(super) fn parse_simple_stmt(&mut self) -> Result<Stmt, ParseError> {
        if let Some(name) = self.peek_increment_name()? {
            self.bump();
            self.expect_punctuation("`++`", |kind| matches!(kind, TokenKind::PlusPlus))?;
            return Ok(Stmt::Increment { name });
        }

        if let Some(name) = self.peek_decrement_name()? {
            self.bump();
            self.expect_punctuation("`--`", |kind| matches!(kind, TokenKind::MinusMinus))?;
            return Ok(Stmt::Decrement { name });
        }

        let first = self.parse_expr()?;
        if !self.check(|kind| matches!(kind, TokenKind::Comma)) {
            if self.check(|kind| matches!(kind, TokenKind::Arrow)) {
                self.bump();
                let value = self.parse_expr()?;
                return Ok(Stmt::Send { chan: first, value });
            }
            if self.check(|kind| matches!(kind, TokenKind::ColonEqual)) {
                self.expect_punctuation("`:=`", |kind| matches!(kind, TokenKind::ColonEqual))?;
                let values = self.parse_expr_list()?;
                return build_short_var_stmt(first, values);
            }
            if self.check(|kind| matches!(kind, TokenKind::Equal)) {
                self.expect_punctuation("`=`", |kind| matches!(kind, TokenKind::Equal))?;
                let values = self.parse_expr_list()?;
                return build_assign_stmt(first, values);
            }
            if let Some(op) = self.peek_compound_assign_op() {
                self.bump();
                let right = self.parse_expr()?;
                let target = assignment_target_from_expr(first.clone())?;
                let value = Expr::Binary {
                    left: Box::new(first),
                    op,
                    right: Box::new(right),
                };
                return Ok(Stmt::Assign { target, value });
            }
            return Ok(Stmt::Expr(first));
        }

        let mut exprs = vec![first];
        while self.check(|kind| matches!(kind, TokenKind::Comma)) {
            self.bump();
            exprs.push(self.parse_expr()?);
        }
        if !(2..=4).contains(&exprs.len()) {
            return Err(ParseError::UnexpectedToken {
                expected: "two to four assignment targets".into(),
                found: format!("{} targets", exprs.len()),
            });
        }

        if self.check(|kind| matches!(kind, TokenKind::ColonEqual)) {
            self.expect_punctuation("`:=`", |kind| matches!(kind, TokenKind::ColonEqual))?;
            let values = self.parse_expr_list()?;
            return build_short_var_multi_stmt(exprs, values);
        }
        if self.check(|kind| matches!(kind, TokenKind::Equal)) {
            self.expect_punctuation("`=`", |kind| matches!(kind, TokenKind::Equal))?;
            let values = self.parse_expr_list()?;
            return build_assign_multi_stmt(exprs, values);
        }

        Err(ParseError::UnexpectedToken {
            expected: "`:=` or `=`".into(),
            found: describe_token(&self.current_token()?.kind),
        })
    }

    fn parse_expr_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut exprs = vec![self.parse_expr()?];
        while self.check(|kind| matches!(kind, TokenKind::Comma)) {
            self.bump();
            exprs.push(self.parse_expr()?);
        }
        Ok(exprs)
    }
}

fn build_short_var_stmt(target: Expr, values: Vec<Expr>) -> Result<Stmt, ParseError> {
    let name = short_var_name_from_expr(target)?;
    match values.as_slice() {
        [value] => Ok(Stmt::ShortVarDecl {
            name,
            value: value.clone(),
        }),
        _ => Ok(Stmt::ShortVarDeclList {
            names: vec![name],
            values,
        }),
    }
}

fn build_assign_stmt(target: Expr, values: Vec<Expr>) -> Result<Stmt, ParseError> {
    let target = assignment_target_from_expr(target)?;
    match values.as_slice() {
        [value] => Ok(Stmt::Assign {
            target,
            value: value.clone(),
        }),
        _ => Ok(Stmt::AssignList {
            targets: vec![target],
            values,
        }),
    }
}

fn build_short_var_multi_stmt(exprs: Vec<Expr>, values: Vec<Expr>) -> Result<Stmt, ParseError> {
    match exprs.as_slice() {
        [first, second] => build_short_var_multi_stmt_from_names(
            vec![
                short_var_name_from_expr(first.clone())?,
                short_var_name_from_expr(second.clone())?,
            ],
            values,
        ),
        [first, second, third] => build_short_var_multi_stmt_from_names(
            vec![
                short_var_name_from_expr(first.clone())?,
                short_var_name_from_expr(second.clone())?,
                short_var_name_from_expr(third.clone())?,
            ],
            values,
        ),
        [first, second, third, fourth] => build_short_var_multi_stmt_from_names(
            vec![
                short_var_name_from_expr(first.clone())?,
                short_var_name_from_expr(second.clone())?,
                short_var_name_from_expr(third.clone())?,
                short_var_name_from_expr(fourth.clone())?,
            ],
            values,
        ),
        _ => Err(ParseError::UnexpectedToken {
            expected: "two to four identifiers".into(),
            found: format!("{} expressions", exprs.len()),
        }),
    }
}

fn build_short_var_multi_stmt_from_names(
    names: Vec<String>,
    values: Vec<Expr>,
) -> Result<Stmt, ParseError> {
    match (names.as_slice(), values.as_slice()) {
        ([first, second], [value]) => Ok(Stmt::ShortVarDeclPair {
            first: first.clone(),
            second: second.clone(),
            value: value.clone(),
        }),
        ([first, second, third], [value]) => Ok(Stmt::ShortVarDeclTriple {
            first: first.clone(),
            second: second.clone(),
            third: third.clone(),
            value: value.clone(),
        }),
        ([first, second, third, fourth], [value]) => Ok(Stmt::ShortVarDeclQuad {
            first: first.clone(),
            second: second.clone(),
            third: third.clone(),
            fourth: fourth.clone(),
            value: value.clone(),
        }),
        _ => Ok(Stmt::ShortVarDeclList { names, values }),
    }
}

fn build_assign_multi_stmt(exprs: Vec<Expr>, values: Vec<Expr>) -> Result<Stmt, ParseError> {
    match exprs.as_slice() {
        [first, second] => build_assign_multi_stmt_from_targets(
            vec![
                assignment_target_from_expr(first.clone())?,
                assignment_target_from_expr(second.clone())?,
            ],
            values,
        ),
        [first, second, third] => build_assign_multi_stmt_from_targets(
            vec![
                assignment_target_from_expr(first.clone())?,
                assignment_target_from_expr(second.clone())?,
                assignment_target_from_expr(third.clone())?,
            ],
            values,
        ),
        [first, second, third, fourth] => build_assign_multi_stmt_from_targets(
            vec![
                assignment_target_from_expr(first.clone())?,
                assignment_target_from_expr(second.clone())?,
                assignment_target_from_expr(third.clone())?,
                assignment_target_from_expr(fourth.clone())?,
            ],
            values,
        ),
        _ => Err(ParseError::UnexpectedToken {
            expected: "two to four assignment targets".into(),
            found: format!("{} expressions", exprs.len()),
        }),
    }
}

fn build_assign_multi_stmt_from_targets(
    targets: Vec<AssignTarget>,
    values: Vec<Expr>,
) -> Result<Stmt, ParseError> {
    match (targets.as_slice(), values.as_slice()) {
        ([first, second], [value]) => Ok(Stmt::AssignPair {
            first: first.clone(),
            second: second.clone(),
            value: value.clone(),
        }),
        ([first, second, third], [value]) => Ok(Stmt::AssignTriple {
            first: first.clone(),
            second: second.clone(),
            third: third.clone(),
            value: value.clone(),
        }),
        ([first, second, third, fourth], [value]) => Ok(Stmt::AssignQuad {
            first: first.clone(),
            second: second.clone(),
            third: third.clone(),
            fourth: fourth.clone(),
            value: value.clone(),
        }),
        _ => Ok(Stmt::AssignList { targets, values }),
    }
}

fn assignment_target_from_expr(expr: Expr) -> Result<AssignTarget, ParseError> {
    match expr {
        Expr::Ident(name) => Ok(AssignTarget::Ident(name)),
        Expr::Unary { op, expr } => match (op, *expr) {
            (UnaryOp::Deref, Expr::Ident(target)) => Ok(AssignTarget::Deref { target }),
            (_, other) => Err(ParseError::UnexpectedToken {
                expected: "assignable dereference target".into(),
                found: format!("{other:?}"),
            }),
        },
        Expr::Selector { receiver, field } => match *receiver {
            Expr::Ident(receiver) => Ok(AssignTarget::Selector { receiver, field }),
            Expr::Unary {
                op: UnaryOp::Deref,
                expr,
            } => match *expr {
                Expr::Ident(target) => Ok(AssignTarget::DerefSelector { target, field }),
                other => Err(ParseError::UnexpectedToken {
                    expected: "assignable dereference selector target".into(),
                    found: format!("{other:?}"),
                }),
            },
            other => Err(ParseError::UnexpectedToken {
                expected: "assignable selector target".into(),
                found: format!("{other:?}"),
            }),
        },
        Expr::Index { target, index } => match *target {
            Expr::Ident(target) => Ok(AssignTarget::Index {
                target,
                index: *index,
            }),
            Expr::Unary {
                op: UnaryOp::Deref,
                expr,
            } => match *expr {
                Expr::Ident(target) => Ok(AssignTarget::DerefIndex {
                    target,
                    index: *index,
                }),
                other => Err(ParseError::UnexpectedToken {
                    expected: "assignable dereference index target".into(),
                    found: format!("{other:?}"),
                }),
            },
            other => Err(ParseError::UnexpectedToken {
                expected: "assignable index target".into(),
                found: format!("{other:?}"),
            }),
        },
        other => Err(ParseError::UnexpectedToken {
            expected: "assignment target".into(),
            found: format!("{other:?}"),
        }),
    }
}

fn short_var_name_from_expr(expr: Expr) -> Result<String, ParseError> {
    match expr {
        Expr::Ident(name) => Ok(name),
        other => Err(ParseError::UnexpectedToken {
            expected: "identifier".into(),
            found: format!("{other:?}"),
        }),
    }
}
