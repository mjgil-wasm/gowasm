use super::{
    float_arg, float_pair_args, int_arg, invalid_math_argument, Float64, Program, Value, ValueData,
    Vm, VmError,
};

pub(super) fn math_inf(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let sign = int_arg(vm, program, "math.Inf", args)?;
    Ok(Value::float(if sign >= 0 {
        f64::INFINITY
    } else {
        f64::NEG_INFINITY
    }))
}

pub(super) fn math_nan(
    _vm: &mut Vm,
    _program: &Program,
    _args: &[Value],
) -> Result<Value, VmError> {
    Ok(Value::float(f64::NAN))
}

pub(super) fn math_is_nan(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.IsNaN", args)?;
    Ok(Value::bool(v.is_nan()))
}

pub(super) fn math_is_inf(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::Float(Float64(f)) = &args[0].data else {
        return Err(invalid_math_argument(
            vm,
            program,
            "math.IsInf",
            "a float64 and int argument",
        )?);
    };
    let ValueData::Int(sign) = &args[1].data else {
        return Err(invalid_math_argument(
            vm,
            program,
            "math.IsInf",
            "a float64 and int argument",
        )?);
    };
    let result = match sign.signum() {
        1 => *f == f64::INFINITY,
        -1 => *f == f64::NEG_INFINITY,
        _ => f.is_infinite(),
    };
    Ok(Value::bool(result))
}

pub(super) fn math_asin(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Asin", args)?;
    Ok(Value::float(v.asin()))
}

pub(super) fn math_acos(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Acos", args)?;
    Ok(Value::float(v.acos()))
}

pub(super) fn math_atan(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Atan", args)?;
    Ok(Value::float(v.atan()))
}

pub(super) fn math_sinh(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Sinh", args)?;
    Ok(Value::float(v.sinh()))
}

pub(super) fn math_cosh(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Cosh", args)?;
    Ok(Value::float(v.cosh()))
}

pub(super) fn math_tanh(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Tanh", args)?;
    Ok(Value::float(v.tanh()))
}

pub(super) fn math_remainder(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (x, y) = float_pair_args(vm, program, "math.Remainder", args)?;
    Ok(Value::float(ieee_remainder(x, y)))
}

fn ieee_remainder(x: f64, y: f64) -> f64 {
    if y == 0.0 || x.is_infinite() || y.is_nan() {
        return f64::NAN;
    }
    let r = x % y;
    if r.abs() == y.abs() * 0.5 {
        if (x / y).round() % 2.0 == 0.0 {
            r
        } else {
            r - y.copysign(x)
        }
    } else if r.abs() > y.abs() * 0.5 {
        r - y.copysign(x)
    } else {
        r
    }
}

pub(super) fn math_dim(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (x, y) = float_pair_args(vm, program, "math.Dim", args)?;
    Ok(Value::float(if x > y { x - y } else { 0.0 }))
}

pub(super) fn math_copysign(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let (x, y) = float_pair_args(vm, program, "math.Copysign", args)?;
    Ok(Value::float(x.copysign(y)))
}

pub(super) fn math_signbit(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Signbit", args)?;
    Ok(Value::bool(v.is_sign_negative()))
}

pub(super) fn math_ldexp(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::Float(Float64(frac)) = &args[0].data else {
        return Err(invalid_math_argument(
            vm,
            program,
            "math.Ldexp",
            "a float64 and int argument",
        )?);
    };
    let ValueData::Int(exp) = &args[1].data else {
        return Err(invalid_math_argument(
            vm,
            program,
            "math.Ldexp",
            "a float64 and int argument",
        )?);
    };
    Ok(Value::float(frac * (2.0_f64).powi(*exp as i32)))
}

pub(super) fn math_expm1(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Expm1", args)?;
    Ok(Value::float(v.exp_m1()))
}

pub(super) fn math_log1p(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Log1p", args)?;
    Ok(Value::float(v.ln_1p()))
}

pub(super) fn math_frexp(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let v = float_arg(vm, program, "math.Frexp", args)?;
    let (frac, exp) = frexp_parts(v);
    Ok(vec![Value::float(frac), Value::int(exp)])
}

fn frexp_parts(v: f64) -> (f64, i64) {
    if v == 0.0 || v.is_infinite() || v.is_nan() {
        return (v, 0);
    }
    let bits = v.to_bits();
    let sign = bits & 0x8000_0000_0000_0000;
    let exp_bits = ((bits >> 52) & 0x7ff) as i64;
    let mantissa = bits & 0x000f_ffff_ffff_ffff;

    if exp_bits == 0 {
        let (frac, exp) = frexp_parts(v * 4_503_599_627_370_496.0);
        return (frac, exp - 52);
    }

    let frac = f64::from_bits(sign | mantissa | 0x3fe0_0000_0000_0000);
    (frac, exp_bits - 1022)
}

pub(super) fn math_cbrt(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Cbrt", args)?;
    Ok(Value::float(v.cbrt()))
}

pub(super) fn math_float64_bits(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Float64bits", args)?;
    Ok(Value::int(v.to_bits() as i64))
}

pub(super) fn math_float64_from_bits(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let ValueData::Int(bits) = &args[0].data else {
        return Err(invalid_math_argument(
            vm,
            program,
            "math.Float64frombits",
            "a uint64 argument",
        )?);
    };
    Ok(Value::float(f64::from_bits(*bits as u64)))
}

pub(super) fn math_logb(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Logb", args)?;
    if v == 0.0 {
        Ok(Value::float(f64::NEG_INFINITY))
    } else if v.is_infinite() {
        Ok(Value::float(f64::INFINITY))
    } else if v.is_nan() {
        Ok(Value::float(f64::NAN))
    } else {
        let (_, exp) = frexp_parts(v);
        Ok(Value::float((exp - 1) as f64))
    }
}

pub(super) fn math_ilogb(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Ilogb", args)?;
    if v == 0.0 {
        Ok(Value::int(-2147483648))
    } else if v.is_infinite() {
        Ok(Value::int(2147483647))
    } else if v.is_nan() {
        Ok(Value::int(-2147483648))
    } else {
        let (_, exp) = frexp_parts(v);
        Ok(Value::int(exp - 1))
    }
}

pub(super) fn math_modf(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let v = float_arg(vm, program, "math.Modf", args)?;
    let integer = v.trunc();
    let mut frac = v - integer;
    if frac == 0.0 {
        frac = 0.0_f64.copysign(v);
    }
    Ok(vec![Value::float(integer), Value::float(frac)])
}
