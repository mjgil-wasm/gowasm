use super::super::time_impl;
use crate::{Program, Value, ValueData, Vm, VmError};

const HTTP_TIME_FORMAT: &str = "Mon, 02 Jan 2006 15:04:05 GMT";

pub(crate) fn parse_time(
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

    let ValueData::String(text) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "http.ParseTime".into(),
            expected: "a string argument".into(),
        });
    };

    let mut last_result = vec![Value::nil(), Value::nil()];
    for layout in [
        HTTP_TIME_FORMAT,
        time_impl::RFC850_LAYOUT,
        time_impl::ANSIC_LAYOUT,
    ] {
        let result = time_impl::time_parse(
            vm,
            program,
            &[Value::string(layout), Value::string(text.clone())],
        )?;
        if matches!(result.get(1).map(|value| &value.data), Some(ValueData::Nil)) {
            return Ok(result);
        }
        last_result = result;
    }

    Ok(last_result)
}
