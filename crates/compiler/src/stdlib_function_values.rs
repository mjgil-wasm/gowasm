use super::*;
use crate::types::format_function_type;
use gowasm_parser::{Expr, Parameter, Stmt};
use gowasm_vm::stdlib_packages;

pub(crate) fn imported_stdlib_selector_function_type(
    package: &str,
    symbol: &str,
) -> Option<String> {
    let (param_types, result_types) = imported_stdlib_selector_signature(package, symbol)?;
    Some(format_function_type(&param_types, &result_types))
}

pub(crate) fn imported_stdlib_selector_value_type(package: &str, symbol: &str) -> Option<String> {
    resolve_stdlib_value(package, symbol).map(|value| value.typ.to_string())
}

pub(crate) fn is_registered_stdlib_package(package: &str) -> bool {
    stdlib_packages()
        .iter()
        .any(|candidate| candidate.name == package)
}

pub(crate) fn unsupported_stdlib_selector_detail(short_name: &str, symbol: &str) -> String {
    format!("package selector `{short_name}.{symbol}` is not supported in the current subset")
}

fn imported_stdlib_selector_signature(
    package: &str,
    symbol: &str,
) -> Option<(Vec<String>, Vec<String>)> {
    let function = resolve_stdlib_function(package, symbol)?;
    if stdlib_function_variadic_param_type(function).is_some() {
        return None;
    }
    let param_types = stdlib_function_param_types(function)?
        .iter()
        .map(|typ| (*typ).to_string())
        .collect::<Vec<_>>();
    let result_types = stdlib_function_result_types(function)?
        .iter()
        .map(|typ| (*typ).to_string())
        .collect::<Vec<_>>();
    Some((param_types, result_types))
}

impl FunctionBuilder<'_> {
    pub(super) fn compile_imported_package_selector(
        &mut self,
        dst: usize,
        short_name: &str,
        package: &str,
        symbol: &str,
    ) -> Result<(), CompileError> {
        if let Some(constant) = resolve_stdlib_constant(package, symbol) {
            self.load_stdlib_selector_value(
                dst,
                constant.typ,
                StdlibValueInit::Constant(constant.value),
            )?;
            return Ok(());
        }
        if let Some(value) = resolve_stdlib_value(package, symbol) {
            self.load_stdlib_selector_value(dst, value.typ, value.value)?;
            return Ok(());
        }
        if let Some(global) = self.lookup_imported_global(package, symbol) {
            self.emitter
                .code
                .push(Instruction::LoadGlobal { dst, global });
            return Ok(());
        }
        if let Some(function) = self.lookup_imported_function_id(package, symbol) {
            self.emitter.code.push(Instruction::MakeClosure {
                dst,
                concrete_type: self
                    .lookup_imported_function_type(package, symbol)
                    .map(|typ| self.lower_runtime_concrete_type(typ))
                    .transpose()?,
                function,
                captures: Vec::new(),
            });
            return Ok(());
        }

        let Some((param_types, result_types)) = imported_stdlib_selector_signature(package, symbol)
        else {
            return Err(CompileError::Unsupported {
                detail: format!(
                    "package selector `{short_name}.{symbol}` cannot be used in value position"
                ),
            });
        };

        let params = param_types
            .iter()
            .enumerate()
            .map(|(index, typ)| Parameter {
                name: format!("arg{index}"),
                typ: typ.clone(),
                variadic: false,
            })
            .collect::<Vec<_>>();
        let call = Expr::Call {
            callee: Box::new(Expr::Selector {
                receiver: Box::new(Expr::Ident(short_name.to_string())),
                field: symbol.to_string(),
            }),
            type_args: Vec::new(),
            args: params
                .iter()
                .map(|parameter| Expr::Ident(parameter.name.clone()))
                .collect(),
        };
        let body = if result_types.is_empty() {
            vec![Stmt::Expr(call)]
        } else {
            vec![Stmt::Return(vec![call])]
        };
        self.compile_function_literal(dst, &params, &result_types, &body)
    }

    fn load_stdlib_selector_value(
        &mut self,
        dst: usize,
        typ: &str,
        value: StdlibValueInit,
    ) -> Result<(), CompileError> {
        match value {
            StdlibValueInit::Constant(value) => self.load_stdlib_constant_value(dst, value),
            StdlibValueInit::NewPointer(type_name) => {
                let zero = self.alloc_register();
                self.compile_zero_value(zero, type_name)?;
                self.emitter.code.push(Instruction::BoxHeap {
                    dst,
                    src: zero,
                    typ: self.pointer_runtime_type(type_name),
                });
            }
            StdlibValueInit::NewPointerWithIntField {
                type_name,
                field,
                value,
            } => {
                let zero = self.alloc_register();
                self.compile_zero_value(zero, type_name)?;
                let field_value = self.alloc_register();
                self.load_stdlib_constant_value(field_value, StdlibConstantValue::Int(value));
                self.emitter.code.push(Instruction::SetField {
                    target: zero,
                    field: field.to_string(),
                    src: field_value,
                });
                self.emitter.code.push(Instruction::BoxHeap {
                    dst,
                    src: zero,
                    typ: self.pointer_runtime_type(type_name),
                });
            }
        }
        if let Some(alias) = self.instantiated_alias_type(typ) {
            self.emitter.code.push(Instruction::Retag {
                dst,
                src: dst,
                typ: alias.type_id,
            });
        }
        Ok(())
    }

    fn load_stdlib_constant_value(&mut self, dst: usize, value: StdlibConstantValue) {
        match value {
            StdlibConstantValue::Int(value) => {
                self.emitter.code.push(Instruction::LoadInt { dst, value });
            }
            StdlibConstantValue::Float(value) => {
                self.emitter
                    .code
                    .push(Instruction::LoadFloat { dst, value });
            }
            StdlibConstantValue::Bool(value) => {
                self.emitter.code.push(Instruction::LoadBool { dst, value });
            }
            StdlibConstantValue::String(value) => self.emitter.code.push(Instruction::LoadString {
                dst,
                value: value.to_string(),
            }),
            StdlibConstantValue::Error(value) => {
                self.emitter.code.push(Instruction::LoadString {
                    dst,
                    value: value.to_string(),
                });
                let function = resolve_stdlib_function("errors", "New")
                    .expect("errors.New should be registered");
                self.emitter.code.push(Instruction::CallStdlib {
                    function,
                    args: vec![dst],
                    dst: Some(dst),
                });
            }
        }
    }
}
