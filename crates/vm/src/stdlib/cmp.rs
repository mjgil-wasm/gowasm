use super::{StdlibFunction, CMP_COMPARE, CMP_LESS, CMP_OR};
use crate::{Program, Value, ValueData, Vm, VmError};

pub(super) const CMP_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: CMP_COMPARE,
        symbol: "Compare",
        returns_value: true,
        handler: cmp_compare,
    },
    StdlibFunction {
        id: CMP_LESS,
        symbol: "Less",
        returns_value: true,
        handler: cmp_less,
    },
    StdlibFunction {
        id: CMP_OR,
        symbol: "Or",
        returns_value: true,
        handler: cmp_or,
    },
];

fn cmp_compare(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let result = compare_ordered(&args[0], &args[1], vm, program)?;
    Ok(Value::int(result))
}

fn cmp_less(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let result = compare_ordered(&args[0], &args[1], vm, program)?;
    Ok(Value::bool(result < 0))
}

fn cmp_or(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.is_empty() {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: 0,
        });
    }
    for arg in args {
        if !is_zero_value(arg) {
            return Ok(arg.clone());
        }
    }
    Ok(args.last().unwrap().clone())
}

fn compare_ordered(a: &Value, b: &Value, vm: &mut Vm, program: &Program) -> Result<i64, VmError> {
    match (&a.data, &b.data) {
        (ValueData::Int(x), ValueData::Int(y)) => Ok(ordering_to_int((*x).cmp(y))),
        (ValueData::Float(x), ValueData::Float(y)) => Ok(ordering_to_int(
            x.0.partial_cmp(&y.0).unwrap_or(std::cmp::Ordering::Equal),
        )),
        (ValueData::String(x), ValueData::String(y)) => Ok(ordering_to_int(x.cmp(y))),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "cmp.Compare".into(),
            expected: "ordered type arguments (int, float64, or string)".into(),
        }),
    }
}

fn ordering_to_int(ord: std::cmp::Ordering) -> i64 {
    match ord {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

fn is_zero_value(v: &Value) -> bool {
    match &v.data {
        ValueData::Int(n) => *n == 0,
        ValueData::Float(f) => f.0 == 0.0,
        ValueData::String(s) => s.is_empty(),
        ValueData::Bool(b) => !*b,
        ValueData::Nil => true,
        _ => false,
    }
}
