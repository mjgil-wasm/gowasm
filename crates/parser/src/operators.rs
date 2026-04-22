use super::*;

impl Parser {
    pub(super) fn parse_add_op(&mut self) -> Option<BinaryOp> {
        let op = match self.tokens.get(self.cursor).map(|token| &token.kind) {
            Some(TokenKind::Plus) => BinaryOp::Add,
            Some(TokenKind::Minus) => BinaryOp::Subtract,
            Some(TokenKind::BitOr) => BinaryOp::BitOr,
            Some(TokenKind::Caret) => BinaryOp::BitXor,
            _ => return None,
        };
        self.bump();
        Some(op)
    }

    pub(super) fn parse_multiply_op(&mut self) -> Option<BinaryOp> {
        let op = match self.tokens.get(self.cursor).map(|token| &token.kind) {
            Some(TokenKind::Star) => BinaryOp::Multiply,
            Some(TokenKind::Slash) => BinaryOp::Divide,
            Some(TokenKind::Percent) => BinaryOp::Modulo,
            Some(TokenKind::BitAnd) => BinaryOp::BitAnd,
            Some(TokenKind::BitClear) => BinaryOp::BitClear,
            Some(TokenKind::ShiftLeft) => BinaryOp::ShiftLeft,
            Some(TokenKind::ShiftRight) => BinaryOp::ShiftRight,
            _ => return None,
        };
        self.bump();
        Some(op)
    }

    pub(super) fn parse_comparison_op(&mut self) -> Option<BinaryOp> {
        let op = match self.tokens.get(self.cursor).map(|token| &token.kind) {
            Some(TokenKind::EqualEqual) => BinaryOp::Equal,
            Some(TokenKind::BangEqual) => BinaryOp::NotEqual,
            Some(TokenKind::Less) => BinaryOp::Less,
            Some(TokenKind::LessEqual) => BinaryOp::LessEqual,
            Some(TokenKind::Greater) => BinaryOp::Greater,
            Some(TokenKind::GreaterEqual) => BinaryOp::GreaterEqual,
            _ => return None,
        };
        self.bump();
        Some(op)
    }
}
