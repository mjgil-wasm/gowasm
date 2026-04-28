use std::collections::HashSet;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResolvedFieldSelector {
    pub(super) path: Vec<String>,
    pub(super) typ: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MethodSelectorCandidate {
    path: Vec<String>,
    function: usize,
}

enum FieldSelectorResolution {
    Missing,
    Unique(ResolvedFieldSelector),
    Ambiguous,
}

impl FunctionBuilder<'_> {
    pub(super) fn resolve_field_selector(
        &self,
        receiver_type: &str,
        field: &str,
    ) -> Result<Option<ResolvedFieldSelector>, CompileError> {
        if self.field_exists_but_is_inaccessible(receiver_type, field) {
            return Err(self.selector_rejects_unexported_field(receiver_type, field));
        }
        let struct_type_name = receiver_type.strip_prefix('*').unwrap_or(receiver_type);
        let mut visited = HashSet::new();
        match self.resolve_field_selector_inner(struct_type_name, field, &mut visited) {
            FieldSelectorResolution::Missing => Ok(None),
            FieldSelectorResolution::Unique(resolution) => Ok(Some(resolution)),
            FieldSelectorResolution::Ambiguous => Err(CompileError::Unsupported {
                detail: format!(
                    "ambiguous promoted field selector `{receiver_type}.{field}` in the current subset"
                ),
            }),
        }
    }

    pub(super) fn ambiguous_method_selector_detail(
        &self,
        receiver: &Expr,
        method: &str,
    ) -> Option<String> {
        self.concrete_method_selector_is_ambiguous(receiver, method)
            .then(|| self.infer_expr_type_name(receiver))
            .flatten()
            .map(|receiver_type| {
                format!(
                    "ambiguous promoted method selector `{receiver_type}.{method}` in the current subset"
                )
            })
    }

    pub(super) fn concrete_method_selector_is_ambiguous(
        &self,
        receiver: &Expr,
        method: &str,
    ) -> bool {
        let Some(receiver_type) = self.infer_expr_type_name(receiver) else {
            return false;
        };
        let key = format!("{receiver_type}.{method}");
        if self.instantiated_method_function_id(&key).is_some()
            && self.instantiated_promoted_method_binding(&key).is_none()
        {
            return false;
        }
        if self.receiver_uses_implicit_address_of(receiver, method) {
            let pointer_key = format!("*{receiver_type}.{method}");
            if self.instantiated_method_function_id(&pointer_key).is_some()
                && self
                    .instantiated_promoted_method_binding(&pointer_key)
                    .is_none()
            {
                return false;
            }
        }
        if parse_pointer_type(&receiver_type).is_some_and(|inner| {
            let inner_key = format!("{inner}.{method}");
            self.instantiated_method_function_id(&inner_key).is_some()
                && self
                    .instantiated_promoted_method_binding(&inner_key)
                    .is_none()
        }) {
            return false;
        }

        self.minimal_method_selector_candidates(&receiver_type, method)
            .len()
            > 1
            || self
                .receiver_uses_implicit_address_of(receiver, method)
                .then(|| {
                    self.minimal_method_selector_candidates(&format!("*{receiver_type}"), method)
                })
                .is_some_and(|candidates| candidates.len() > 1)
    }

    fn resolve_field_selector_inner(
        &self,
        struct_type_name: &str,
        field: &str,
        visited: &mut HashSet<String>,
    ) -> FieldSelectorResolution {
        if !visited.insert(struct_type_name.to_string()) {
            return FieldSelectorResolution::Missing;
        }

        let resolution = self
            .instantiated_struct_type(struct_type_name)
            .map(|struct_type| {
                if let Some(field_def) = struct_type.fields.iter().find(|candidate| {
                    candidate.name == field
                        && self.field_is_accessible_from_current_package(
                            struct_type_name,
                            &candidate.name,
                        )
                }) {
                    return FieldSelectorResolution::Unique(ResolvedFieldSelector {
                        path: vec![field.to_string()],
                        typ: field_def.typ.clone(),
                    });
                }

                let mut matches = Vec::new();
                for decl in &struct_type.fields {
                    if !decl.embedded
                        || !self
                            .field_is_accessible_from_current_package(struct_type_name, &decl.name)
                    {
                        continue;
                    }
                    let embedded_type = decl.typ.strip_prefix('*').unwrap_or(&decl.typ);
                    if let FieldSelectorResolution::Unique(mut resolution) =
                        self.resolve_field_selector_inner(embedded_type, field, visited)
                    {
                        resolution.path.insert(0, decl.name.clone());
                        matches.push(resolution);
                    }
                }
                shortest_unique_field_resolution(matches)
            })
            .unwrap_or(FieldSelectorResolution::Missing);

        visited.remove(struct_type_name);
        resolution
    }

    fn minimal_method_selector_candidates(
        &self,
        receiver_type: &str,
        method: &str,
    ) -> Vec<MethodSelectorCandidate> {
        let mut visited = HashSet::new();
        let mut candidates =
            self.collect_method_selector_candidates(receiver_type, method, &mut visited);
        let Some(min_depth) = candidates
            .iter()
            .map(|candidate| candidate.path.len())
            .min()
        else {
            return Vec::new();
        };
        candidates.retain(|candidate| candidate.path.len() == min_depth);
        let mut unique = Vec::new();
        let mut seen = HashSet::new();
        for candidate in candidates {
            let key = format!("{}:{}", candidate.function, candidate.path.join("."));
            if seen.insert(key) {
                unique.push(candidate);
            }
        }
        unique
    }

    fn collect_method_selector_candidates(
        &self,
        receiver_type: &str,
        method: &str,
        visited: &mut HashSet<String>,
    ) -> Vec<MethodSelectorCandidate> {
        if !visited.insert(receiver_type.to_string()) {
            return Vec::new();
        }

        let direct = self
            .direct_method_selector_candidate(receiver_type, method)
            .into_iter()
            .collect::<Vec<_>>();
        if !direct.is_empty() {
            visited.remove(receiver_type);
            return direct;
        }

        let pointer_receiver = receiver_type.starts_with('*');
        let struct_type_name = receiver_type.strip_prefix('*').unwrap_or(receiver_type);
        let mut candidates = Vec::new();
        if let Some(struct_type) = self.instantiated_struct_type(struct_type_name) {
            for field in &struct_type.fields {
                if !field.embedded {
                    continue;
                }
                for source_receiver in promoted_source_receivers(&field.typ, pointer_receiver) {
                    for mut candidate in
                        self.collect_method_selector_candidates(&source_receiver, method, visited)
                    {
                        candidate.path.insert(0, field.name.clone());
                        candidates.push(candidate);
                    }
                }
            }
        }

        visited.remove(receiver_type);
        candidates
    }

    fn direct_method_selector_candidate(
        &self,
        receiver_type: &str,
        method: &str,
    ) -> Option<MethodSelectorCandidate> {
        let key = format!("{receiver_type}.{method}");
        let function = self.instantiated_method_function_id(&key)?;
        self.instantiated_promoted_method_binding(&key)
            .is_none()
            .then_some(MethodSelectorCandidate {
                path: Vec::new(),
                function,
            })
    }
}

fn shortest_unique_field_resolution(
    matches: Vec<ResolvedFieldSelector>,
) -> FieldSelectorResolution {
    let Some(min_depth) = matches.iter().map(|candidate| candidate.path.len()).min() else {
        return FieldSelectorResolution::Missing;
    };
    let mut shortest = matches
        .into_iter()
        .filter(|candidate| candidate.path.len() == min_depth);
    let Some(first) = shortest.next() else {
        return FieldSelectorResolution::Missing;
    };
    if shortest.next().is_some() {
        FieldSelectorResolution::Ambiguous
    } else {
        FieldSelectorResolution::Unique(first)
    }
}

fn promoted_source_receivers(field_type: &str, pointer_receiver: bool) -> Vec<String> {
    let embedded_type = field_type.strip_prefix('*').unwrap_or(field_type);
    if field_type.starts_with('*') {
        vec![embedded_type.to_string(), field_type.to_string()]
    } else if pointer_receiver {
        vec![embedded_type.to_string(), format!("*{embedded_type}")]
    } else {
        vec![embedded_type.to_string()]
    }
}
