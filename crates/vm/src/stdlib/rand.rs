use super::*;
use crate::value::ValueData;

pub(super) const RAND_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: RAND_INTN,
        symbol: "Intn",
        returns_value: true,
        handler: rand_intn,
    },
    StdlibFunction {
        id: RAND_FLOAT64,
        symbol: "Float64",
        returns_value: true,
        handler: rand_float64,
    },
    StdlibFunction {
        id: RAND_INT,
        symbol: "Int",
        returns_value: true,
        handler: rand_int,
    },
    StdlibFunction {
        id: RAND_SEED,
        symbol: "Seed",
        returns_value: false,
        handler: rand_seed,
    },
    StdlibFunction {
        id: RAND_INT63,
        symbol: "Int63",
        returns_value: true,
        handler: rand_int63,
    },
    StdlibFunction {
        id: RAND_INT63N,
        symbol: "Int63n",
        returns_value: true,
        handler: rand_int63n,
    },
    StdlibFunction {
        id: RAND_INT31,
        symbol: "Int31",
        returns_value: true,
        handler: rand_int31,
    },
    StdlibFunction {
        id: RAND_INT31N,
        symbol: "Int31n",
        returns_value: true,
        handler: rand_int31n,
    },
];

const RNG_A: u64 = 6364136223846793005;
const RNG_C: u64 = 1442695040888963407;

fn rng_next(vm: &mut Vm) -> u64 {
    vm.rng_state = vm.rng_state.wrapping_mul(RNG_A).wrapping_add(RNG_C);
    vm.record_rng_advance(vm.rng_state);
    vm.rng_state
}

fn rng_int63(vm: &mut Vm) -> i64 {
    (rng_next(vm) >> 1) as i64
}

fn int_arg_rand(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &[Value],
) -> Result<i64, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    match &args[0].data {
        ValueData::Int(v) => Ok(*v),
        _ => Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "an int argument".into(),
        }),
    }
}

fn rand_intn(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let n = int_arg_rand(vm, program, "rand.Intn", args)?;
    if n <= 0 {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "rand.Intn".into(),
            expected: "a positive int argument".into(),
        });
    }
    Ok(Value::int((rng_int63(vm) % n).abs()))
}

fn rand_float64(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if !args.is_empty() {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 0,
            actual: args.len(),
        });
    }
    Ok(Value::float((rng_int63(vm) as f64) / ((1u64 << 63) as f64)))
}

fn rand_int(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if !args.is_empty() {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 0,
            actual: args.len(),
        });
    }
    Ok(Value::int(rng_int63(vm)))
}

fn rand_seed(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let seed = int_arg_rand(vm, program, "rand.Seed", args)?;
    vm.rng_state = seed as u64;
    vm.record_rng_seed(vm.rng_state);
    Ok(Value::nil())
}

fn rand_int63(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if !args.is_empty() {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 0,
            actual: args.len(),
        });
    }
    Ok(Value::int(rng_int63(vm)))
}

fn rand_int63n(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let n = int_arg_rand(vm, program, "rand.Int63n", args)?;
    if n <= 0 {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "rand.Int63n".into(),
            expected: "a positive int argument".into(),
        });
    }
    Ok(Value::int(rng_int63(vm) % n))
}

fn rand_int31(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if !args.is_empty() {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 0,
            actual: args.len(),
        });
    }
    Ok(Value::int((rng_int63(vm) >> 32) as i32 as i64 & 0x7FFFFFFF))
}

fn rand_int31n(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let n = int_arg_rand(vm, program, "rand.Int31n", args)?;
    if n <= 0 {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "rand.Int31n".into(),
            expected: "a positive int argument".into(),
        });
    }
    let val = ((rng_int63(vm) >> 32) as i32 as i64 & 0x7FFFFFFF) % n;
    Ok(Value::int(val))
}
