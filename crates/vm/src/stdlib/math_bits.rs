use super::{
    StdlibFunction, MATH_BITS_LEADING_ZEROS, MATH_BITS_LEN, MATH_BITS_ONES_COUNT,
    MATH_BITS_REVERSE, MATH_BITS_REVERSE_BYTES, MATH_BITS_ROTATE_LEFT, MATH_BITS_TRAILING_ZEROS,
};
use crate::{Program, Value, ValueData, Vm, VmError};

pub(super) const MATH_BITS_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: MATH_BITS_ONES_COUNT,
        symbol: "OnesCount",
        returns_value: true,
        handler: math_bits_ones_count,
    },
    StdlibFunction {
        id: MATH_BITS_LEADING_ZEROS,
        symbol: "LeadingZeros",
        returns_value: true,
        handler: math_bits_leading_zeros,
    },
    StdlibFunction {
        id: MATH_BITS_TRAILING_ZEROS,
        symbol: "TrailingZeros",
        returns_value: true,
        handler: math_bits_trailing_zeros,
    },
    StdlibFunction {
        id: MATH_BITS_LEN,
        symbol: "Len",
        returns_value: true,
        handler: math_bits_len,
    },
    StdlibFunction {
        id: MATH_BITS_ROTATE_LEFT,
        symbol: "RotateLeft",
        returns_value: true,
        handler: math_bits_rotate_left,
    },
    StdlibFunction {
        id: MATH_BITS_REVERSE,
        symbol: "Reverse",
        returns_value: true,
        handler: math_bits_reverse,
    },
    StdlibFunction {
        id: MATH_BITS_REVERSE_BYTES,
        symbol: "ReverseBytes",
        returns_value: true,
        handler: math_bits_reverse_bytes,
    },
];

fn uint_arg(vm: &mut Vm, program: &Program, builtin: &str, args: &[Value]) -> Result<u64, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    match &args[0].data {
        ValueData::Int(v) => Ok(*v as u64),
        _ => Err(invalid_bits_argument(
            vm,
            program,
            builtin,
            "a uint argument",
        )?),
    }
}

fn invalid_bits_argument(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    expected: &str,
) -> Result<VmError, VmError> {
    Ok(VmError::InvalidStringFunctionArgument {
        function: vm.current_function_name(program)?,
        builtin: builtin.into(),
        expected: expected.into(),
    })
}

fn math_bits_ones_count(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = uint_arg(vm, program, "bits.OnesCount", args)?;
    Ok(Value::int(v.count_ones() as i64))
}

fn math_bits_leading_zeros(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let v = uint_arg(vm, program, "bits.LeadingZeros", args)?;
    Ok(Value::int(v.leading_zeros() as i64))
}

fn math_bits_trailing_zeros(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let v = uint_arg(vm, program, "bits.TrailingZeros", args)?;
    Ok(Value::int(v.trailing_zeros() as i64))
}

fn math_bits_len(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = uint_arg(vm, program, "bits.Len", args)?;
    Ok(Value::int(if v == 0 {
        0
    } else {
        64 - v.leading_zeros() as i64
    }))
}

fn math_bits_rotate_left(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::Int(x) = &args[0].data else {
        return Err(invalid_bits_argument(
            vm,
            program,
            "bits.RotateLeft",
            "a uint and int argument",
        )?);
    };
    let ValueData::Int(k) = &args[1].data else {
        return Err(invalid_bits_argument(
            vm,
            program,
            "bits.RotateLeft",
            "a uint and int argument",
        )?);
    };
    let x = *x as u64;
    let k = *k as i32;
    Ok(Value::int(x.rotate_left(k as u32) as i64))
}

fn math_bits_reverse(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = uint_arg(vm, program, "bits.Reverse", args)?;
    Ok(Value::int(v.reverse_bits() as i64))
}

fn math_bits_reverse_bytes(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let v = uint_arg(vm, program, "bits.ReverseBytes", args)?;
    Ok(Value::int(v.swap_bytes() as i64))
}
