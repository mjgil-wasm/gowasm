use super::{StdlibFunction, SHA1_SUM};
use crate::{Program, Value, ValueData, Vm, VmError};

pub(super) const SHA1_FUNCTIONS: &[StdlibFunction] = &[StdlibFunction {
    id: SHA1_SUM,
    symbol: "Sum",
    returns_value: true,
    handler: sha1_sum,
}];

fn sha1_sum(vm: &mut Vm, program: &Program, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::WrongArgumentCount {
            function: vm.current_function_name(program)?,
            expected: 1,
            actual: args.len(),
        });
    }
    let bytes = extract_byte_slice(vm, program, &args[0])?;
    let hash = compute_sha1(&bytes);
    let values: Vec<Value> = hash.iter().map(|&b| Value::int(b as i64)).collect();
    Ok(Value::array(values))
}

fn extract_byte_slice(vm: &mut Vm, program: &Program, value: &Value) -> Result<Vec<u8>, VmError> {
    let ValueData::Slice(slice) = &value.data else {
        return Err(VmError::InvalidStringFunctionArgument {
            function: vm.current_function_name(program)?,
            builtin: "sha1.Sum".into(),
            expected: "a []byte argument".into(),
        });
    };
    slice
        .values_snapshot()
        .iter()
        .map(|v| match v.data {
            ValueData::Int(b) => Ok(b as u8),
            _ => Err(VmError::InvalidStringFunctionArgument {
                function: "sha1".into(),
                builtin: "sha1.Sum".into(),
                expected: "byte element".into(),
            }),
        })
        .collect()
}

fn compute_sha1(data: &[u8]) -> [u8; 20] {
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;

    let bit_len = (data.len() as u64) * 8;
    let mut padded = data.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let (mut a, mut b, mut c, mut d, mut e) = (h0, h1, h2, h3, h4);
        for (i, w_i) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1u32),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDCu32),
                _ => (b ^ c ^ d, 0xCA62C1D6u32),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*w_i);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let mut result = [0u8; 20];
    result[0..4].copy_from_slice(&h0.to_be_bytes());
    result[4..8].copy_from_slice(&h1.to_be_bytes());
    result[8..12].copy_from_slice(&h2.to_be_bytes());
    result[12..16].copy_from_slice(&h3.to_be_bytes());
    result[16..20].copy_from_slice(&h4.to_be_bytes());
    result
}

pub(super) const SHA1_BLOCK_SIZE: i64 = 64;
pub(super) const SHA1_SIZE: i64 = 20;
