use super::super::{
    net_url_impl, unsupported_multi_result_stdlib, StdlibFunction, StdlibMethod,
    NET_HTTP_RESPONSE_LOCATION,
};
use super::core::canonicalize_header_key_text;
use crate::{MapValue, Program, Value, ValueData, Vm, VmError, TYPE_HTTP_RESPONSE, TYPE_URL_PTR};

pub(super) const RESPONSE_URL_FIELD: &str = "__response_url";

pub(crate) const NET_HTTP_RESPONSE_METHODS: &[StdlibMethod] = &[StdlibMethod {
    receiver_type: "*http.Response",
    method: "Location",
    function: NET_HTTP_RESPONSE_LOCATION,
}];

pub(crate) const NET_HTTP_RESPONSE_METHOD_FUNCTIONS: &[StdlibFunction] = &[StdlibFunction {
    id: NET_HTTP_RESPONSE_LOCATION,
    symbol: "Location",
    returns_value: false,
    handler: unsupported_multi_result_stdlib,
}];

pub(crate) fn response_location(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let response = response_receiver_value(vm, program, "(*http.Response).Location", &args[0])?;
    let ValueData::Struct(fields) = &response.data else {
        return Err(invalid_response_argument(
            vm,
            program,
            "(*http.Response).Location",
        ));
    };

    let Some(location) = response_location_header(
        vm,
        program,
        "(*http.Response).Location",
        response_field(vm, program, "(*http.Response).Location", fields, "Header")?,
    )?
    else {
        return Ok(vec![
            Value::nil_pointer(TYPE_URL_PTR),
            Value::error("http: no Location header in response"),
        ]);
    };

    if let Some(base_url) = response_hidden_url(fields) {
        let parsed_base =
            net_url_impl::url_parse(vm, program, &[Value::string(base_url.to_string())])?;
        if matches!(
            parsed_base.get(1).map(|value| &value.data),
            Some(ValueData::Nil)
        ) {
            return net_url_impl::url_url_parse(
                vm,
                program,
                &[parsed_base[0].clone(), Value::string(location.to_string())],
            );
        }
    }

    net_url_impl::url_parse(vm, program, &[Value::string(location.to_string())])
}

fn response_location_header(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    header: &Value,
) -> Result<Option<String>, VmError> {
    let ValueData::Map(map) = &header.data else {
        return Err(invalid_response_argument(vm, program, builtin));
    };
    header_first_value(vm, program, builtin, map, "Location")
}

fn response_hidden_url(fields: &[(String, Value)]) -> Option<&str> {
    let (_, value) = fields
        .iter()
        .find(|(field_name, _)| field_name == RESPONSE_URL_FIELD)?;
    let ValueData::String(text) = &value.data else {
        return None;
    };
    if text.is_empty() {
        return None;
    }
    Some(text)
}

fn response_field<'a>(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    fields: &'a [(String, Value)],
    name: &str,
) -> Result<&'a Value, VmError> {
    fields
        .iter()
        .find(|(field_name, _)| field_name == name)
        .map(|(_, value)| value)
        .ok_or_else(|| invalid_response_argument(vm, program, builtin))
}

fn response_receiver_value(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    receiver: &Value,
) -> Result<Value, VmError> {
    let response = vm.deref_pointer(program, receiver)?;
    if response.typ != TYPE_HTTP_RESPONSE {
        return Err(invalid_response_argument(vm, program, builtin));
    }
    let ValueData::Struct(_) = &response.data else {
        return Err(invalid_response_argument(vm, program, builtin));
    };
    Ok(response)
}

fn header_first_value(
    vm: &Vm,
    program: &Program,
    builtin: &str,
    header: &MapValue,
    name: &str,
) -> Result<Option<String>, VmError> {
    let canonical = canonicalize_header_key_text(name);
    let Some(values) = header.get(&Value::string(canonical)) else {
        return Ok(None);
    };
    let ValueData::Slice(values) = &values.data else {
        return Err(invalid_response_argument(vm, program, builtin));
    };
    let visible = values.values_snapshot();
    let Some(first) = visible.first() else {
        return Ok(None);
    };
    let ValueData::String(text) = &first.data else {
        return Err(invalid_response_argument(vm, program, builtin));
    };
    Ok(Some(text.clone()))
}

fn invalid_response_argument(vm: &Vm, program: &Program, builtin: &str) -> VmError {
    VmError::InvalidStringFunctionArgument {
        function: vm
            .current_function_name(program)
            .unwrap_or_else(|_| "<engine>".into()),
        builtin: builtin.into(),
        expected: "a valid *http.Response receiver".into(),
    }
}
