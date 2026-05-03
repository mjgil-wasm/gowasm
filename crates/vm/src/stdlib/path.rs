use super::{
    StdlibFunction, PATH_BASE, PATH_CLEAN, PATH_DIR, PATH_EXT, PATH_IS_ABS, PATH_JOIN, PATH_MATCH,
    PATH_SPLIT,
};
use crate::{Program, Value, ValueData, Vm, VmError};

pub(super) const PATH_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: PATH_BASE,
        symbol: "Base",
        returns_value: true,
        handler: path_base,
    },
    StdlibFunction {
        id: PATH_CLEAN,
        symbol: "Clean",
        returns_value: true,
        handler: path_clean,
    },
    StdlibFunction {
        id: PATH_DIR,
        symbol: "Dir",
        returns_value: true,
        handler: path_dir,
    },
    StdlibFunction {
        id: PATH_EXT,
        symbol: "Ext",
        returns_value: true,
        handler: path_ext,
    },
    StdlibFunction {
        id: PATH_IS_ABS,
        symbol: "IsAbs",
        returns_value: true,
        handler: path_is_abs,
    },
    StdlibFunction {
        id: PATH_SPLIT,
        symbol: "Split",
        returns_value: false,
        handler: unsupported_path_multi_result,
    },
    StdlibFunction {
        id: PATH_JOIN,
        symbol: "Join",
        returns_value: true,
        handler: path_join,
    },
    StdlibFunction {
        id: PATH_MATCH,
        symbol: "Match",
        returns_value: false,
        handler: unsupported_path_multi_result,
    },
];

pub(super) fn path_base(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let path = path_arg(vm, program, "path.Base", args)?;
    Ok(Value::string(base(path)))
}

pub(super) fn path_clean(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let path = path_arg(vm, program, "path.Clean", args)?;
    Ok(Value::string(clean(path)))
}

pub(super) fn path_dir(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let path = path_arg(vm, program, "path.Dir", args)?;
    Ok(Value::string(dir(path)))
}

pub(super) fn path_ext(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let path = path_arg(vm, program, "path.Ext", args)?;
    Ok(Value::string(ext(path)))
}

pub(super) fn path_is_abs(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Value, VmError> {
    let path = path_arg(vm, program, "path.IsAbs", args)?;
    Ok(Value::bool(is_abs(path)))
}

pub(super) fn path_split(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let path = path_arg(vm, program, "path.Split", args)?;
    let (dir, file) = split(path);
    Ok(vec![Value::string(dir), Value::string(file)])
}

pub(super) fn path_join(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    let paths = path_string_args(vm, program, "path.Join", args)?;
    Ok(Value::string(join(&paths)))
}

pub(super) fn path_match(
    vm: &mut Vm,
    program: &Program,
    args: &[Value],
) -> Result<Vec<Value>, VmError> {
    let (pattern, name) = path_pair_args(vm, program, "path.Match", args)?;
    match match_pattern(pattern, name) {
        Ok(matched) => Ok(vec![Value::bool(matched), Value::nil()]),
        Err(detail) => Ok(vec![Value::bool(false), Value::error(detail)]),
    }
}

fn path_arg<'a>(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &'a [Value],
) -> Result<&'a str, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }

    let ValueData::String(path) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "a string argument".into(),
        });
    };
    Ok(path)
}

fn path_string_args<'a>(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &'a [Value],
) -> Result<Vec<&'a str>, VmError> {
    let mut paths = Vec::with_capacity(args.len());
    for arg in args {
        let ValueData::String(path) = &arg.data else {
            return Err(VmError::InvalidStringFunctionArgument {
                function: vm.current_function_name(program)?,
                builtin: builtin.into(),
                expected: "string arguments".into(),
            });
        };
        paths.push(path.as_str());
    }
    Ok(paths)
}

fn path_pair_args<'a>(
    vm: &mut Vm,
    program: &Program,
    builtin: &str,
    args: &'a [Value],
) -> Result<(&'a str, &'a str), VmError> {
    if args.len() != 2 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 2,
            actual: args.len(),
        });
    }

    let ValueData::String(left) = &args[0].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string arguments".into(),
        });
    };
    let ValueData::String(right) = &args[1].data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: builtin.into(),
            expected: "string arguments".into(),
        });
    };
    Ok((left, right))
}

pub(super) fn clean(path: &str) -> String {
    if path.is_empty() {
        return ".".into();
    }

    let input = path.as_bytes();
    let rooted = input[0] == b'/';
    let mut output = Vec::with_capacity(input.len());
    let mut read = 0;
    let mut dotdot = 0;

    if rooted {
        output.push(b'/');
        read = 1;
        dotdot = 1;
    }

    while read < input.len() {
        match () {
            _ if input[read] == b'/' => {
                read += 1;
            }
            _ if input[read] == b'.' && (read + 1 == input.len() || input[read + 1] == b'/') => {
                read += 1;
            }
            _ if input[read] == b'.'
                && read + 1 < input.len()
                && input[read + 1] == b'.'
                && (read + 2 == input.len() || input[read + 2] == b'/') =>
            {
                read += 2;
                if output.len() > dotdot {
                    output.pop();
                    while output.len() > dotdot && output.last() != Some(&b'/') {
                        output.pop();
                    }
                    if output.len() > dotdot && output.last() == Some(&b'/') {
                        output.pop();
                    }
                } else if !rooted {
                    if !output.is_empty() {
                        output.push(b'/');
                    }
                    output.push(b'.');
                    output.push(b'.');
                    dotdot = output.len();
                }
            }
            _ => {
                if (rooted && output.len() != 1) || (!rooted && !output.is_empty()) {
                    output.push(b'/');
                }
                while read < input.len() && input[read] != b'/' {
                    output.push(input[read]);
                    read += 1;
                }
            }
        }
    }

    if output.is_empty() {
        return ".".into();
    }

    String::from_utf8(output).expect("cleaned path should stay utf-8")
}

pub(super) fn base(path: &str) -> String {
    if path.is_empty() {
        return ".".into();
    }

    let mut path = path;
    while path.ends_with('/') {
        path = &path[..path.len() - 1];
    }
    if let Some(index) = path.rfind('/') {
        path = &path[index + 1..];
    }
    if path.is_empty() {
        return "/".into();
    }
    path.into()
}

pub(super) fn dir(path: &str) -> String {
    clean(split(path).0)
}

pub(super) fn ext(path: &str) -> String {
    let bytes = path.as_bytes();
    for index in (0..bytes.len()).rev() {
        if bytes[index] == b'/' {
            break;
        }
        if bytes[index] == b'.' {
            return path[index..].into();
        }
    }
    String::new()
}

pub(super) fn is_abs(path: &str) -> bool {
    path.starts_with('/')
}

pub(super) fn split(path: &str) -> (&str, &str) {
    let split_at = path.rfind('/').map(|index| index + 1).unwrap_or(0);
    (&path[..split_at], &path[split_at..])
}

pub(super) fn join(paths: &[&str]) -> String {
    let size = paths.iter().map(|path| path.len()).sum::<usize>();
    if size == 0 {
        return String::new();
    }

    let mut joined = String::with_capacity(size + paths.len().saturating_sub(1));
    for path in paths {
        if !joined.is_empty() || !path.is_empty() {
            if !joined.is_empty() {
                joined.push('/');
            }
            joined.push_str(path);
        }
    }

    clean(&joined)
}

const BAD_PATTERN: &str = "syntax error in pattern";

pub(super) fn match_pattern(pattern: &str, name: &str) -> Result<bool, &'static str> {
    let mut pattern = pattern;
    let mut name = name;

    'pattern: while !pattern.is_empty() {
        let (star, chunk, rest) = scan_chunk(pattern);
        pattern = rest;
        if star && chunk.is_empty() {
            return Ok(!name.as_bytes().contains(&b'/'));
        }

        let (tail, ok) = match_chunk(chunk, name)?;
        if ok && (tail.is_empty() || !pattern.is_empty()) {
            name = tail;
            continue;
        }

        if star {
            for next_index in star_skip_indices(name) {
                let (tail, ok) = match_chunk(chunk, &name[next_index..])?;
                if ok {
                    if pattern.is_empty() && !tail.is_empty() {
                        continue;
                    }
                    name = tail;
                    continue 'pattern;
                }
            }
        }

        while !pattern.is_empty() {
            let (_, chunk, rest) = scan_chunk(pattern);
            pattern = rest;
            let _ = match_chunk(chunk, "")?;
        }
        return Ok(false);
    }

    Ok(name.is_empty())
}

fn scan_chunk(pattern: &str) -> (bool, &str, &str) {
    let bytes = pattern.as_bytes();
    let mut start = 0;
    let mut star = false;
    while start < bytes.len() && bytes[start] == b'*' {
        start += 1;
        star = true;
    }

    let mut in_range = false;
    let mut end = start;
    while end < bytes.len() {
        match bytes[end] {
            b'\\' => {
                end += 1;
                if end < bytes.len() {
                    end += 1;
                }
            }
            b'[' => {
                in_range = true;
                end += 1;
            }
            b']' => {
                in_range = false;
                end += 1;
            }
            b'*' if !in_range => break,
            _ => {
                end += 1;
            }
        }
    }

    (star, &pattern[start..end], &pattern[end..])
}

fn match_chunk<'a>(chunk: &str, name: &'a str) -> Result<(&'a str, bool), &'static str> {
    let chunk_bytes = chunk.as_bytes();
    let name_bytes = name.as_bytes();
    let mut chunk_index = 0;
    let mut name_index = 0;
    let mut failed = false;

    while chunk_index < chunk_bytes.len() {
        if !failed && name_index == name_bytes.len() {
            failed = true;
        }

        match chunk_bytes[chunk_index] {
            b'[' => {
                let mut current = '\0';
                if !failed {
                    let (rune, next_index) = next_rune(name, name_index);
                    current = rune;
                    name_index = next_index;
                }
                chunk_index += 1;

                let mut negated = false;
                if chunk_index < chunk_bytes.len() && chunk_bytes[chunk_index] == b'^' {
                    negated = true;
                    chunk_index += 1;
                }

                let mut matched = false;
                let mut ranges = 0;
                loop {
                    if chunk_index < chunk_bytes.len()
                        && chunk_bytes[chunk_index] == b']'
                        && ranges > 0
                    {
                        chunk_index += 1;
                        break;
                    }

                    let (lo, next_index) = get_escaped_char(chunk, chunk_index)?;
                    chunk_index = next_index;
                    let mut hi = lo;
                    if chunk_index < chunk_bytes.len() && chunk_bytes[chunk_index] == b'-' {
                        let (range_hi, next_index) = get_escaped_char(chunk, chunk_index + 1)?;
                        hi = range_hi;
                        chunk_index = next_index;
                    }
                    if !failed && lo <= current && current <= hi {
                        matched = true;
                    }
                    ranges += 1;
                }

                if matched == negated {
                    failed = true;
                }
            }
            b'?' => {
                if !failed {
                    if name_bytes[name_index] == b'/' {
                        failed = true;
                    } else {
                        let (_, next_index) = next_rune(name, name_index);
                        name_index = next_index;
                    }
                }
                chunk_index += 1;
            }
            b'\\' => {
                chunk_index += 1;
                if chunk_index == chunk_bytes.len() {
                    return Err(BAD_PATTERN);
                }
                if !failed {
                    if chunk_bytes[chunk_index] != name_bytes[name_index] {
                        failed = true;
                    } else {
                        name_index += 1;
                    }
                }
                chunk_index += 1;
            }
            _ => {
                if !failed {
                    if chunk_bytes[chunk_index] != name_bytes[name_index] {
                        failed = true;
                    } else {
                        name_index += 1;
                    }
                }
                chunk_index += 1;
            }
        }
    }

    if failed {
        Ok(("", false))
    } else {
        Ok((&name[name_index..], true))
    }
}

fn get_escaped_char(chunk: &str, mut index: usize) -> Result<(char, usize), &'static str> {
    let bytes = chunk.as_bytes();
    if index >= bytes.len() || matches!(bytes[index], b'-' | b']') {
        return Err(BAD_PATTERN);
    }
    if bytes[index] == b'\\' {
        index += 1;
        if index >= bytes.len() {
            return Err(BAD_PATTERN);
        }
    }

    let (rune, next_index) = next_rune(chunk, index);
    if next_index == chunk.len() {
        return Err(BAD_PATTERN);
    }
    Ok((rune, next_index))
}

fn next_rune(text: &str, index: usize) -> (char, usize) {
    let rune = text[index..]
        .chars()
        .next()
        .expect("caller must not read past the end of the string");
    (rune, index + rune.len_utf8())
}

fn star_skip_indices(name: &str) -> Vec<usize> {
    let mut indices = Vec::new();
    for (index, rune) in name.char_indices() {
        if rune == '/' {
            break;
        }
        indices.push(index + rune.len_utf8());
    }
    indices
}

fn unsupported_path_multi_result(
    _vm: &mut Vm,
    _program: &Program,
    _args: &[Value],
) -> Result<Value, VmError> {
    Ok(Value::nil())
}
