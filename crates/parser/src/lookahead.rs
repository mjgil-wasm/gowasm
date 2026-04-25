use super::*;

impl Parser {
    pub(super) fn try_consume_label(&mut self) -> Option<String> {
        if let Ok(token) = self.current_token() {
            if let TokenKind::Ident(name) = &token.kind {
                let label = name.clone();
                self.bump();
                return Some(label);
            }
        }
        None
    }

    pub(super) fn peek_labeled_stmt(&self) -> Result<Option<String>, ParseError> {
        let current = self.current_token()?;
        let Some(next) = self.tokens.get(self.cursor + 1) else {
            return Ok(None);
        };
        if let (TokenKind::Ident(name), TokenKind::Colon) = (&current.kind, &next.kind) {
            Ok(Some(name.clone()))
        } else {
            Ok(None)
        }
    }

    pub(super) fn peek_increment_name(&self) -> Result<Option<String>, ParseError> {
        let current = self.current_token()?;
        let Some(next) = self.tokens.get(self.cursor + 1) else {
            return Ok(None);
        };

        if let (TokenKind::Ident(name), TokenKind::PlusPlus) = (&current.kind, &next.kind) {
            Ok(Some(name.clone()))
        } else {
            Ok(None)
        }
    }

    pub(super) fn peek_decrement_name(&self) -> Result<Option<String>, ParseError> {
        let current = self.current_token()?;
        let Some(next) = self.tokens.get(self.cursor + 1) else {
            return Ok(None);
        };

        if let (TokenKind::Ident(name), TokenKind::MinusMinus) = (&current.kind, &next.kind) {
            Ok(Some(name.clone()))
        } else {
            Ok(None)
        }
    }

    pub(super) fn peek_compound_assign_op(&self) -> Option<BinaryOp> {
        let kind = &self.tokens.get(self.cursor)?.kind;
        match kind {
            TokenKind::PlusEqual => Some(BinaryOp::Add),
            TokenKind::MinusEqual => Some(BinaryOp::Subtract),
            TokenKind::StarEqual => Some(BinaryOp::Multiply),
            TokenKind::SlashEqual => Some(BinaryOp::Divide),
            TokenKind::PercentEqual => Some(BinaryOp::Modulo),
            TokenKind::BitAndEqual => Some(BinaryOp::BitAnd),
            TokenKind::BitOrEqual => Some(BinaryOp::BitOr),
            TokenKind::CaretEqual => Some(BinaryOp::BitXor),
            TokenKind::ShiftLeftEqual => Some(BinaryOp::ShiftLeft),
            TokenKind::ShiftRightEqual => Some(BinaryOp::ShiftRight),
            _ => None,
        }
    }

    pub(super) fn peek_type_switch(&self) -> Option<Option<String>> {
        let start = self.cursor;
        let current = self.tokens.get(start)?;
        let binding = if let TokenKind::Ident(name) = &current.kind {
            if let Some(next) = self.tokens.get(start + 1) {
                if matches!(next.kind, TokenKind::ColonEqual) {
                    Some(name.clone())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        for i in start..self.tokens.len() {
            let kind = &self.tokens[i].kind;
            if matches!(kind, TokenKind::LBrace | TokenKind::Semicolon) {
                break;
            }
            if matches!(kind, TokenKind::Dot) {
                if let (Some(lp), Some(ty), Some(rp)) = (
                    self.tokens.get(i + 1),
                    self.tokens.get(i + 2),
                    self.tokens.get(i + 3),
                ) {
                    if matches!(lp.kind, TokenKind::LParen)
                        && matches!(ty.kind, TokenKind::Type)
                        && matches!(rp.kind, TokenKind::RParen)
                    {
                        return Some(binding);
                    }
                }
            }
        }
        None
    }
}
