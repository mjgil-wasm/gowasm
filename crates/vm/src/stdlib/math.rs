use super::{
    StdlibFunction, MATH_ABS, MATH_ACOS, MATH_ASIN, MATH_ATAN, MATH_ATAN2, MATH_CBRT, MATH_CEIL,
    MATH_COPYSIGN, MATH_COS, MATH_COSH, MATH_DIM, MATH_EXP, MATH_EXPM1, MATH_FLOAT64_BITS,
    MATH_FLOAT64_FROM_BITS, MATH_FLOOR, MATH_FREXP, MATH_HYPOT, MATH_ILOGB, MATH_INF, MATH_IS_INF,
    MATH_IS_NAN, MATH_LDEXP, MATH_LOG, MATH_LOG10, MATH_LOG1P, MATH_LOG2, MATH_LOGB, MATH_MAX,
    MATH_MIN, MATH_MOD, MATH_MODF, MATH_NAN, MATH_POW, MATH_REMAINDER, MATH_ROUND, MATH_SIGNBIT,
    MATH_SIN, MATH_SINH, MATH_SQRT, MATH_TAN, MATH_TANH, MATH_TRUNC,
};
use crate::{Float64, Program, Value, ValueData, Vm, VmError};

#[path = "math/special.rs"]
mod special;
use special::*;

pub(super) const MATH_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: MATH_ABS,
        symbol: "Abs",
        returns_value: true,
        handler: math_abs,
    },
    StdlibFunction {
        id: MATH_CEIL,
        symbol: "Ceil",
        returns_value: true,
        handler: math_ceil,
    },
    StdlibFunction {
        id: MATH_FLOOR,
        symbol: "Floor",
        returns_value: true,
        handler: math_floor,
    },
    StdlibFunction {
        id: MATH_MAX,
        symbol: "Max",
        returns_value: true,
        handler: math_max,
    },
    StdlibFunction {
        id: MATH_MIN,
        symbol: "Min",
        returns_value: true,
        handler: math_min,
    },
    StdlibFunction {
        id: MATH_MOD,
        symbol: "Mod",
        returns_value: true,
        handler: math_mod,
    },
    StdlibFunction {
        id: MATH_POW,
        symbol: "Pow",
        returns_value: true,
        handler: math_pow,
    },
    StdlibFunction {
        id: MATH_ROUND,
        symbol: "Round",
        returns_value: true,
        handler: math_round,
    },
    StdlibFunction {
        id: MATH_SQRT,
        symbol: "Sqrt",
        returns_value: true,
        handler: math_sqrt,
    },
    StdlibFunction {
        id: MATH_TRUNC,
        symbol: "Trunc",
        returns_value: true,
        handler: math_trunc,
    },
    StdlibFunction {
        id: MATH_SIN,
        symbol: "Sin",
        returns_value: true,
        handler: math_sin,
    },
    StdlibFunction {
        id: MATH_COS,
        symbol: "Cos",
        returns_value: true,
        handler: math_cos,
    },
    StdlibFunction {
        id: MATH_TAN,
        symbol: "Tan",
        returns_value: true,
        handler: math_tan,
    },
    StdlibFunction {
        id: MATH_LOG,
        symbol: "Log",
        returns_value: true,
        handler: math_log,
    },
    StdlibFunction {
        id: MATH_LOG2,
        symbol: "Log2",
        returns_value: true,
        handler: math_log2,
    },
    StdlibFunction {
        id: MATH_LOG10,
        symbol: "Log10",
        returns_value: true,
        handler: math_log10,
    },
    StdlibFunction {
        id: MATH_EXP,
        symbol: "Exp",
        returns_value: true,
        handler: math_exp,
    },
    StdlibFunction {
        id: MATH_ATAN2,
        symbol: "Atan2",
        returns_value: true,
        handler: math_atan2,
    },
    StdlibFunction {
        id: MATH_HYPOT,
        symbol: "Hypot",
        returns_value: true,
        handler: math_hypot,
    },
    StdlibFunction {
        id: MATH_INF,
        symbol: "Inf",
        returns_value: true,
        handler: math_inf,
    },
    StdlibFunction {
        id: MATH_NAN,
        symbol: "NaN",
        returns_value: true,
        handler: math_nan,
    },
    StdlibFunction {
        id: MATH_IS_NAN,
        symbol: "IsNaN",
        returns_value: true,
        handler: math_is_nan,
    },
    StdlibFunction {
        id: MATH_IS_INF,
        symbol: "IsInf",
        returns_value: true,
        handler: math_is_inf,
    },
    StdlibFunction {
        id: MATH_ASIN,
        symbol: "Asin",
        returns_value: true,
        handler: math_asin,
    },
    StdlibFunction {
        id: MATH_ACOS,
        symbol: "Acos",
        returns_value: true,
        handler: math_acos,
    },
    StdlibFunction {
        id: MATH_ATAN,
        symbol: "Atan",
        returns_value: true,
        handler: math_atan,
    },
    StdlibFunction {
        id: MATH_SINH,
        symbol: "Sinh",
        returns_value: true,
        handler: math_sinh,
    },
    StdlibFunction {
        id: MATH_COSH,
        symbol: "Cosh",
        returns_value: true,
        handler: math_cosh,
    },
    StdlibFunction {
        id: MATH_TANH,
        symbol: "Tanh",
        returns_value: true,
        handler: math_tanh,
    },
    StdlibFunction {
        id: MATH_REMAINDER,
        symbol: "Remainder",
        returns_value: true,
        handler: math_remainder,
    },
    StdlibFunction {
        id: MATH_DIM,
        symbol: "Dim",
        returns_value: true,
        handler: math_dim,
    },
    StdlibFunction {
        id: MATH_COPYSIGN,
        symbol: "Copysign",
        returns_value: true,
        handler: math_copysign,
    },
    StdlibFunction {
        id: MATH_SIGNBIT,
        symbol: "Signbit",
        returns_value: true,
        handler: math_signbit,
    },
    StdlibFunction {
        id: MATH_LDEXP,
        symbol: "Ldexp",
        returns_value: true,
        handler: math_ldexp,
    },
    StdlibFunction {
        id: MATH_FREXP,
        symbol: "Frexp",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
    StdlibFunction {
        id: MATH_EXPM1,
        symbol: "Expm1",
        returns_value: true,
        handler: math_expm1,
    },
    StdlibFunction {
        id: MATH_LOG1P,
        symbol: "Log1p",
        returns_value: true,
        handler: math_log1p,
    },
    StdlibFunction {
        id: MATH_CBRT,
        symbol: "Cbrt",
        returns_value: true,
        handler: math_cbrt,
    },
    StdlibFunction {
        id: MATH_FLOAT64_BITS,
        symbol: "Float64bits",
        returns_value: true,
        handler: math_float64_bits,
    },
    StdlibFunction {
        id: MATH_FLOAT64_FROM_BITS,
        symbol: "Float64frombits",
        returns_value: true,
        handler: math_float64_from_bits,
    },
    StdlibFunction {
        id: MATH_LOGB,
        symbol: "Logb",
        returns_value: true,
        handler: math_logb,
    },
    StdlibFunction {
        id: MATH_ILOGB,
        symbol: "Ilogb",
        returns_value: true,
        handler: math_ilogb,
    },
    StdlibFunction {
        id: MATH_MODF,
        symbol: "Modf",
        returns_value: false,
        handler: super::unsupported_multi_result_stdlib,
    },
];

pub(super) const MATH_CONSTANTS: &[super::StdlibConstant] = &[
    super::StdlibConstant {
        symbol: "Pi",
        typ: "float64",
        value: super::StdlibConstantValue::Float(Float64(std::f64::consts::PI)),
    },
    super::StdlibConstant {
        symbol: "E",
        typ: "float64",
        value: super::StdlibConstantValue::Float(Float64(std::f64::consts::E)),
    },
    super::StdlibConstant {
        symbol: "Phi",
        typ: "float64",
        value: super::StdlibConstantValue::Float(Float64(1.618_033_988_749_895)),
    },
    super::StdlibConstant {
        symbol: "Sqrt2",
        typ: "float64",
        value: super::StdlibConstantValue::Float(Float64(std::f64::consts::SQRT_2)),
    },
    super::StdlibConstant {
        symbol: "Ln2",
        typ: "float64",
        value: super::StdlibConstantValue::Float(Float64(std::f64::consts::LN_2)),
    },
    super::StdlibConstant {
        symbol: "Ln10",
        typ: "float64",
        value: super::StdlibConstantValue::Float(Float64(std::f64::consts::LN_10)),
    },
    super::StdlibConstant {
        symbol: "MaxFloat64",
        typ: "float64",
        value: super::StdlibConstantValue::Float(Float64(f64::MAX)),
    },
    super::StdlibConstant {
        symbol: "SmallestNonzeroFloat64",
        typ: "float64",
        value: super::StdlibConstantValue::Float(Float64(f64::from_bits(1))),
    },
    super::StdlibConstant {
        symbol: "MaxInt",
        typ: "int",
        value: super::StdlibConstantValue::Int(i64::MAX),
    },
    super::StdlibConstant {
        symbol: "MinInt",
        typ: "int",
        value: super::StdlibConstantValue::Int(i64::MIN),
    },
    super::StdlibConstant {
        symbol: "MaxInt8",
        typ: "int",
        value: super::StdlibConstantValue::Int(i8::MAX as i64),
    },
    super::StdlibConstant {
        symbol: "MaxInt16",
        typ: "int",
        value: super::StdlibConstantValue::Int(i16::MAX as i64),
    },
    super::StdlibConstant {
        symbol: "MaxInt32",
        typ: "int",
        value: super::StdlibConstantValue::Int(i32::MAX as i64),
    },
    super::StdlibConstant {
        symbol: "MaxInt64",
        typ: "int",
        value: super::StdlibConstantValue::Int(i64::MAX),
    },
    super::StdlibConstant {
        symbol: "MaxUint8",
        typ: "int",
        value: super::StdlibConstantValue::Int(u8::MAX as i64),
    },
    super::StdlibConstant {
        symbol: "MaxUint16",
        typ: "int",
        value: super::StdlibConstantValue::Int(u16::MAX as i64),
    },
    super::StdlibConstant {
        symbol: "MaxUint32",
        typ: "int",
        value: super::StdlibConstantValue::Int(u32::MAX as i64),
    },
    super::StdlibConstant {
        symbol: "MinInt8",
        typ: "int",
        value: super::StdlibConstantValue::Int(i8::MIN as i64),
    },
    super::StdlibConstant {
        symbol: "MinInt16",
        typ: "int",
        value: super::StdlibConstantValue::Int(i16::MIN as i64),
    },
    super::StdlibConstant {
        symbol: "MinInt32",
        typ: "int",
        value: super::StdlibConstantValue::Int(i32::MIN as i64),
    },
    super::StdlibConstant {
        symbol: "MinInt64",
        typ: "int",
        value: super::StdlibConstantValue::Int(i64::MIN),
    },
    super::StdlibConstant {
        symbol: "MaxFloat32",
        typ: "float64",
        value: super::StdlibConstantValue::Float(Float64(f32::MAX as f64)),
    },
    super::StdlibConstant {
        symbol: "SmallestNonzeroFloat32",
        typ: "float64",
        value: super::StdlibConstantValue::Float(Float64(f32::MIN_POSITIVE as f64)),
    },
];

fn float_arg(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> Result<f64, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    match &args[0].data {
        ValueData::Float(Float64(v)) => Ok(*v),
        _ => Err(invalid_math_argument(
            vm,
            program,
            builtin,
            "a float64 argument",
        )?),
    }
}

fn float_pair_args(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> Result<(f64, f64), VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }
    let ValueData::Float(Float64(left)) = &args[0].data else {
        return Err(invalid_math_argument(
            vm,
            program,
            builtin,
            "two float64 arguments",
        )?);
    };
    let ValueData::Float(Float64(right)) = &args[1].data else {
        return Err(invalid_math_argument(
            vm,
            program,
            builtin,
            "two float64 arguments",
        )?);
    };
    Ok((*left, *right))
}

fn invalid_math_argument(
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

pub(super) fn math_abs(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Abs", args)?;
    Ok(Value::float(v.abs()))
}

pub(super) fn math_ceil(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Ceil", args)?;
    Ok(Value::float(v.ceil()))
}

pub(super) fn math_floor(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Floor", args)?;
    Ok(Value::float(v.floor()))
}

pub(super) fn math_max(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (a, b) = float_pair_args(vm, program, "math.Max", args)?;
    Ok(Value::float(go_math_max(a, b)))
}

pub(super) fn math_min(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (a, b) = float_pair_args(vm, program, "math.Min", args)?;
    Ok(Value::float(go_math_min(a, b)))
}

fn go_math_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    if a == 0.0 && b == 0.0 {
        if a.is_sign_negative() && b.is_sign_negative() {
            return -0.0;
        }
        return 0.0;
    }
    if a > b {
        a
    } else {
        b
    }
}

fn go_math_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    if a == 0.0 && b == 0.0 {
        if a.is_sign_negative() || b.is_sign_negative() {
            return -0.0;
        }
        return 0.0;
    }
    if a < b {
        a
    } else {
        b
    }
}

pub(super) fn math_mod(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (x, y) = float_pair_args(vm, program, "math.Mod", args)?;
    Ok(Value::float(x % y))
}

pub(super) fn math_pow(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (base, exp) = float_pair_args(vm, program, "math.Pow", args)?;
    Ok(Value::float(base.powf(exp)))
}

pub(super) fn math_round(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Round", args)?;
    Ok(Value::float(v.round()))
}

pub(super) fn math_sqrt(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Sqrt", args)?;
    Ok(Value::float(v.sqrt()))
}

pub(super) fn math_trunc(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Trunc", args)?;
    Ok(Value::float(v.trunc()))
}

pub(super) fn math_sin(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Sin", args)?;
    Ok(Value::float(v.sin()))
}

pub(super) fn math_cos(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Cos", args)?;
    Ok(Value::float(v.cos()))
}

pub(super) fn math_tan(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Tan", args)?;
    Ok(Value::float(v.tan()))
}

pub(super) fn math_log(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Log", args)?;
    Ok(Value::float(v.ln()))
}

pub(super) fn math_log2(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Log2", args)?;
    Ok(Value::float(v.log2()))
}

pub(super) fn math_log10(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Log10", args)?;
    Ok(Value::float(v.log10()))
}

pub(super) fn math_exp(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let v = float_arg(vm, program, "math.Exp", args)?;
    Ok(Value::float(v.exp()))
}

pub(super) fn math_atan2(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (y, x) = float_pair_args(vm, program, "math.Atan2", args)?;
    Ok(Value::float(y.atan2(x)))
}

pub(super) fn math_hypot(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let (x, y) = float_pair_args(vm, program, "math.Hypot", args)?;
    Ok(Value::float(x.hypot(y)))
}

fn int_arg(vm: &mut Vm, program: &Program, builtin: &str, args: &[Value]) -> Result<i64, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    match &args[0].data {
        ValueData::Int(v) => Ok(*v),
        _ => Err(invalid_math_argument(
            vm,
            program,
            builtin,
            "an int argument",
        )?),
    }
}

pub(super) fn math_frexp(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    special::math_frexp(vm, program, args)
}

pub(super) fn math_modf(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    special::math_modf(vm, program, args)
}
