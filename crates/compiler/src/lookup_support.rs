use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn current_scope(&self) -> &HashMap<String, usize> {
        self.scopes.current_scope()
    }

    pub(super) fn current_scope_mut(&mut self) -> &mut HashMap<String, usize> {
        self.scopes.current_scope_mut()
    }

    pub(super) fn current_type_scope_mut(&mut self) -> &mut HashMap<String, String> {
        self.scopes.current_type_scope_mut()
    }

    pub(super) fn lookup_local(&self, name: &str) -> Option<usize> {
        self.scopes.lookup_local(name)
    }

    pub(super) fn lookup_global(&self, name: &str) -> Option<usize> {
        self.env.globals.get(name).map(|binding| binding.index)
    }

    pub(super) fn imported_package_symbol_key(package: &str, symbol: &str) -> String {
        format!("{package}.{symbol}")
    }

    pub(super) fn lookup_imported_global_binding(
        &self,
        package: &str,
        symbol: &str,
    ) -> Option<&GlobalBinding> {
        self.env
            .imported_package_tables
            .globals
            .get(&Self::imported_package_symbol_key(package, symbol))
    }

    pub(super) fn lookup_imported_global(&self, package: &str, symbol: &str) -> Option<usize> {
        self.lookup_imported_global_binding(package, symbol)
            .map(|binding| binding.index)
    }

    pub(super) fn lookup_imported_global_type(&self, package: &str, symbol: &str) -> Option<&str> {
        self.lookup_imported_global_binding(package, symbol)
            .and_then(|binding| binding.typ.as_deref())
    }

    pub(super) fn lookup_imported_function_id(&self, package: &str, symbol: &str) -> Option<usize> {
        self.env
            .imported_package_tables
            .function_ids
            .get(&Self::imported_package_symbol_key(package, symbol))
            .copied()
    }

    pub(super) fn lookup_imported_function_type(
        &self,
        package: &str,
        symbol: &str,
    ) -> Option<&str> {
        self.env
            .imported_package_tables
            .function_types
            .get(&Self::imported_package_symbol_key(package, symbol))
            .map(String::as_str)
    }

    pub(super) fn lookup_imported_function_result_types(
        &self,
        package: &str,
        symbol: &str,
    ) -> Option<&Vec<String>> {
        self.env
            .imported_package_tables
            .function_result_types
            .get(&Self::imported_package_symbol_key(package, symbol))
    }

    pub(super) fn imported_function_is_variadic(&self, package: &str, symbol: &str) -> bool {
        self.env
            .imported_package_tables
            .variadic_functions
            .contains(&Self::imported_package_symbol_key(package, symbol))
    }

    pub(super) fn lookup_local_type(&self, name: &str) -> Option<&str> {
        self.scopes
            .type_scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).map(String::as_str))
    }

    pub(super) fn lookup_global_type(&self, name: &str) -> Option<&str> {
        self.env
            .globals
            .get(name)
            .and_then(|binding| binding.typ.as_deref())
    }

    pub(super) fn lookup_interface_type(&self, expr: &Expr) -> Option<InterfaceTypeDef> {
        self.infer_expr_type_name(expr)
            .and_then(|type_name| self.instantiated_interface_type(&type_name))
    }

    pub(super) fn compile_expr_into_with_hint(
        &mut self,
        dst: usize,
        expr: &Expr,
        target_type: Option<&str>,
    ) -> Result<(), CompileError> {
        if let Some(target_type) = target_type {
            self.ensure_runtime_visible_type(target_type)?;
        }
        if matches!(expr, Expr::NilLiteral) {
            return match target_type {
                Some(typ) if self.type_allows_nil(typ) => self.compile_zero_value(dst, typ),
                Some(typ) => Err(CompileError::Unsupported {
                    detail: format!("cannot use `nil` as `{typ}` in the current subset"),
                }),
                None => {
                    self.emitter.code.push(Instruction::LoadNil { dst });
                    Ok(())
                }
            };
        }

        if self.compile_const_expr_value(dst, expr, target_type)? {
            return Ok(());
        }

        self.ensure_expr_runtime_types(expr)?;
        self.compile_expr_into(dst, expr)?;
        if let Some(target_type) = target_type {
            if let Some(alias) = self.instantiated_alias_type(target_type) {
                self.emitter.code.push(Instruction::Retag {
                    dst,
                    src: dst,
                    typ: alias.type_id,
                });
                return Ok(());
            }
            self.maybe_retag_assignable_value(dst, expr, target_type)?;
        }
        Ok(())
    }
}
