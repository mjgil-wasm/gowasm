use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn field_is_accessible_from_current_package(
        &self,
        type_name: &str,
        field: &str,
    ) -> bool {
        imported_type_qualifier(type_name).is_none() || field_is_exported(field)
    }

    pub(super) fn literal_rejects_unexported_field(
        &self,
        type_name: &str,
        field: &str,
    ) -> CompileError {
        CompileError::Unsupported {
            detail: format!(
                "cannot use unexported field `{field}` in imported struct literal `{type_name}`"
            ),
        }
    }

    pub(super) fn selector_rejects_unexported_field(
        &self,
        receiver_type: &str,
        field: &str,
    ) -> CompileError {
        CompileError::Unsupported {
            detail: format!(
                "cannot access unexported field selector `{receiver_type}.{field}` in the current subset"
            ),
        }
    }

    pub(super) fn field_exists_but_is_inaccessible(
        &self,
        receiver_type: &str,
        field: &str,
    ) -> bool {
        let struct_type_name = receiver_type.strip_prefix('*').unwrap_or(receiver_type);
        let Some(struct_type) = self.instantiated_struct_type(struct_type_name) else {
            return false;
        };
        struct_type
            .fields
            .iter()
            .any(|candidate| candidate.name == field)
            && !self.field_is_accessible_from_current_package(struct_type_name, field)
    }
}

fn imported_type_qualifier(type_name: &str) -> Option<&str> {
    let head = type_name.strip_prefix('*').unwrap_or(type_name);
    let head = head.split('[').next().unwrap_or(head);
    head.rsplit_once('.').map(|(qualifier, _)| qualifier)
}

fn field_is_exported(field: &str) -> bool {
    field.chars().next().is_some_and(char::is_uppercase)
}
