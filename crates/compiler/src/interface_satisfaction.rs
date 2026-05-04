use gowasm_parser::InterfaceMethodDecl;

pub(crate) fn method_signatures_match(
    candidate: &InterfaceMethodDecl,
    required: &InterfaceMethodDecl,
) -> bool {
    candidate.name == required.name
        && candidate.result_types == required.result_types
        && candidate.params.len() == required.params.len()
        && candidate
            .params
            .iter()
            .zip(required.params.iter())
            .all(|(candidate, required)| candidate.typ == required.typ)
}

pub(crate) fn first_interface_method_mismatch(
    candidates: &[InterfaceMethodDecl],
    required_methods: &[InterfaceMethodDecl],
) -> Option<String> {
    required_methods.iter().find_map(|required| {
        let same_name = candidates
            .iter()
            .find(|candidate| candidate.name == required.name);
        match same_name {
            Some(candidate) if method_signatures_match(candidate, required) => None,
            Some(candidate) => Some(format!(
                "method `{}` has signature `{}`, want `{}`",
                required.name,
                render_method_signature(candidate),
                render_method_signature(required)
            )),
            None => Some(format!(
                "missing method `{}`",
                render_method_signature(required)
            )),
        }
    })
}

pub(crate) fn first_method_set_mismatch(
    candidate_sets: &[Vec<InterfaceMethodDecl>],
    required_methods: &[InterfaceMethodDecl],
) -> Option<String> {
    required_methods.iter().find_map(|required| {
        let same_name = candidate_sets
            .iter()
            .flat_map(|methods| methods.iter())
            .find(|candidate| candidate.name == required.name);
        match same_name {
            Some(candidate) if method_signatures_match(candidate, required) => None,
            Some(candidate) => Some(format!(
                "method `{}` has signature `{}`, want `{}`",
                required.name,
                render_method_signature(candidate),
                render_method_signature(required)
            )),
            None => Some(format!(
                "missing method `{}`",
                render_method_signature(required)
            )),
        }
    })
}

pub(crate) fn render_method_signature(method: &InterfaceMethodDecl) -> String {
    let params = method
        .params
        .iter()
        .map(|param| param.typ.clone())
        .collect::<Vec<_>>()
        .join(", ");
    let results = match method.result_types.as_slice() {
        [] => String::new(),
        [result] => format!(" {result}"),
        many => format!(" ({})", many.join(", ")),
    };
    format!("{}({params}){results}", method.name, results = results)
}
