use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn compile_int_binary(
        &mut self,
        dst: usize,
        op: BinaryOp,
        left: &Expr,
        right: &Expr,
    ) -> Result<(), CompileError> {
        let left = self.compile_value_expr(left)?;
        let right = self.compile_value_expr(right)?;
        let instruction = match op {
            BinaryOp::Subtract => Instruction::Subtract { dst, left, right },
            BinaryOp::BitOr => Instruction::BitOr { dst, left, right },
            BinaryOp::BitXor => Instruction::BitXor { dst, left, right },
            BinaryOp::BitAnd => Instruction::BitAnd { dst, left, right },
            BinaryOp::BitClear => Instruction::BitClear { dst, left, right },
            BinaryOp::Multiply => Instruction::Multiply { dst, left, right },
            BinaryOp::Divide => Instruction::Divide { dst, left, right },
            BinaryOp::Modulo => Instruction::Modulo { dst, left, right },
            BinaryOp::ShiftLeft => Instruction::ShiftLeft { dst, left, right },
            BinaryOp::ShiftRight => Instruction::ShiftRight { dst, left, right },
            _ => unreachable!("non-integer op should not use compile_int_binary"),
        };
        self.emitter.code.push(instruction);
        Ok(())
    }
}
