use super::resolve_stdlib_function;
use crate::{Function, Instruction, Program, Vm, TYPE_URL, TYPE_URL_PTR};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should exist")
}

#[test]
fn net_url_raw_hint_rendering_falls_back_for_malformed_runtime_values() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 32,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "https".into(),
                },
                Instruction::LoadString {
                    dst: 30,
                    value: String::new(),
                },
                Instruction::LoadBool {
                    dst: 31,
                    value: false,
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "example.com".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "/a/b".into(),
                },
                Instruction::LoadString {
                    dst: 3,
                    value: "/a%2Fb".into(),
                },
                Instruction::LoadString {
                    dst: 4,
                    value: "q=1".into(),
                },
                Instruction::LoadString {
                    dst: 5,
                    value: "frag/part".into(),
                },
                Instruction::LoadString {
                    dst: 6,
                    value: "frag%2Fpart".into(),
                },
                Instruction::MakeStruct {
                    dst: 7,
                    typ: TYPE_URL,
                    fields: vec![
                        ("Scheme".into(), 0),
                        ("Opaque".into(), 30),
                        ("Host".into(), 1),
                        ("Path".into(), 2),
                        ("RawPath".into(), 3),
                        ("ForceQuery".into(), 31),
                        ("RawQuery".into(), 4),
                        ("Fragment".into(), 5),
                        ("RawFragment".into(), 6),
                    ],
                },
                Instruction::BoxHeap {
                    dst: 8,
                    src: 7,
                    typ: TYPE_URL_PTR,
                },
                Instruction::CallMethod {
                    receiver: 8,
                    method: "EscapedPath".into(),
                    args: vec![],
                    dst: Some(9),
                },
                Instruction::CallMethod {
                    receiver: 8,
                    method: "EscapedFragment".into(),
                    args: vec![],
                    dst: Some(10),
                },
                Instruction::CallMethod {
                    receiver: 7,
                    method: "String".into(),
                    args: vec![],
                    dst: Some(11),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![9],
                    dst: None,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![10],
                    dst: None,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![11],
                    dst: None,
                },
                Instruction::CallMethod {
                    receiver: 8,
                    method: "RequestURI".into(),
                    args: vec![],
                    dst: Some(11),
                },
                Instruction::LoadString {
                    dst: 12,
                    value: "/space path".into(),
                },
                Instruction::LoadString {
                    dst: 13,
                    value: "/broken%zz".into(),
                },
                Instruction::LoadString {
                    dst: 14,
                    value: "frag ment".into(),
                },
                Instruction::LoadString {
                    dst: 15,
                    value: "%zz".into(),
                },
                Instruction::MakeStruct {
                    dst: 16,
                    typ: TYPE_URL,
                    fields: vec![
                        ("Scheme".into(), 0),
                        ("Opaque".into(), 30),
                        ("Host".into(), 1),
                        ("Path".into(), 12),
                        ("RawPath".into(), 13),
                        ("ForceQuery".into(), 31),
                        ("RawQuery".into(), 4),
                        ("Fragment".into(), 14),
                        ("RawFragment".into(), 15),
                    ],
                },
                Instruction::BoxHeap {
                    dst: 17,
                    src: 16,
                    typ: TYPE_URL_PTR,
                },
                Instruction::CallMethod {
                    receiver: 17,
                    method: "EscapedPath".into(),
                    args: vec![],
                    dst: Some(18),
                },
                Instruction::CallMethod {
                    receiver: 17,
                    method: "EscapedFragment".into(),
                    args: vec![],
                    dst: Some(19),
                },
                Instruction::CallMethod {
                    receiver: 16,
                    method: "String".into(),
                    args: vec![],
                    dst: Some(20),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![11],
                    dst: None,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![18],
                    dst: None,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![19],
                    dst: None,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![20],
                    dst: None,
                },
                Instruction::CallMethod {
                    receiver: 17,
                    method: "RequestURI".into(),
                    args: vec![],
                    dst: Some(20),
                },
                Instruction::LoadString {
                    dst: 21,
                    value: "/plain".into(),
                },
                Instruction::LoadString {
                    dst: 22,
                    value: "/other".into(),
                },
                Instruction::LoadString {
                    dst: 23,
                    value: "frag".into(),
                },
                Instruction::LoadString {
                    dst: 24,
                    value: "other".into(),
                },
                Instruction::MakeStruct {
                    dst: 25,
                    typ: TYPE_URL,
                    fields: vec![
                        ("Scheme".into(), 0),
                        ("Opaque".into(), 30),
                        ("Host".into(), 1),
                        ("Path".into(), 21),
                        ("RawPath".into(), 22),
                        ("ForceQuery".into(), 31),
                        ("RawQuery".into(), 4),
                        ("Fragment".into(), 23),
                        ("RawFragment".into(), 24),
                    ],
                },
                Instruction::BoxHeap {
                    dst: 26,
                    src: 25,
                    typ: TYPE_URL_PTR,
                },
                Instruction::CallMethod {
                    receiver: 26,
                    method: "EscapedPath".into(),
                    args: vec![],
                    dst: Some(27),
                },
                Instruction::CallMethod {
                    receiver: 26,
                    method: "EscapedFragment".into(),
                    args: vec![],
                    dst: Some(28),
                },
                Instruction::CallMethod {
                    receiver: 25,
                    method: "String".into(),
                    args: vec![],
                    dst: Some(29),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![27],
                    dst: None,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![28],
                    dst: None,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![29],
                    dst: None,
                },
                Instruction::CallMethod {
                    receiver: 26,
                    method: "RequestURI".into(),
                    args: vec![],
                    dst: Some(29),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![20],
                    dst: None,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![29],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("program should run with direct url.URL runtime values");

    assert_eq!(
        vm.stdout(),
        "/a%2Fb\nfrag%2Fpart\nhttps://example.com/a%2Fb?q=1#frag%2Fpart\n/a%2Fb?q=1\n/space%20path\nfrag%20ment\nhttps://example.com/space%20path?q=1#frag%20ment\n/plain\nfrag\nhttps://example.com/plain?q=1#frag\n/space%20path?q=1\n/plain?q=1\n"
    );
}

#[test]
fn net_url_opaque_and_force_query_render_for_runtime_values() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 17,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "mailto".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "dev@example.com".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: String::new(),
                },
                Instruction::LoadBool {
                    dst: 3,
                    value: true,
                },
                Instruction::MakeStruct {
                    dst: 4,
                    typ: TYPE_URL,
                    fields: vec![
                        ("Scheme".into(), 0),
                        ("Opaque".into(), 1),
                        ("Host".into(), 2),
                        ("Path".into(), 2),
                        ("RawPath".into(), 2),
                        ("ForceQuery".into(), 3),
                        ("RawQuery".into(), 2),
                        ("Fragment".into(), 2),
                        ("RawFragment".into(), 2),
                    ],
                },
                Instruction::BoxHeap {
                    dst: 5,
                    src: 4,
                    typ: TYPE_URL_PTR,
                },
                Instruction::CallMethod {
                    receiver: 4,
                    method: "String".into(),
                    args: vec![],
                    dst: Some(6),
                },
                Instruction::CallMethod {
                    receiver: 5,
                    method: "RequestURI".into(),
                    args: vec![],
                    dst: Some(7),
                },
                Instruction::LoadString {
                    dst: 8,
                    value: "custom".into(),
                },
                Instruction::LoadString {
                    dst: 9,
                    value: "//example.com/path".into(),
                },
                Instruction::LoadString {
                    dst: 10,
                    value: "x=1".into(),
                },
                Instruction::LoadBool {
                    dst: 11,
                    value: false,
                },
                Instruction::MakeStruct {
                    dst: 12,
                    typ: TYPE_URL,
                    fields: vec![
                        ("Scheme".into(), 8),
                        ("Opaque".into(), 9),
                        ("Host".into(), 2),
                        ("Path".into(), 2),
                        ("RawPath".into(), 2),
                        ("ForceQuery".into(), 11),
                        ("RawQuery".into(), 10),
                        ("Fragment".into(), 2),
                        ("RawFragment".into(), 2),
                    ],
                },
                Instruction::BoxHeap {
                    dst: 13,
                    src: 12,
                    typ: TYPE_URL_PTR,
                },
                Instruction::CallMethod {
                    receiver: 12,
                    method: "String".into(),
                    args: vec![],
                    dst: Some(14),
                },
                Instruction::CallMethod {
                    receiver: 13,
                    method: "RequestURI".into(),
                    args: vec![],
                    dst: Some(15),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![6],
                    dst: None,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![7],
                    dst: None,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![14],
                    dst: None,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![15],
                    dst: None,
                },
                Instruction::MakeStruct {
                    dst: 16,
                    typ: TYPE_URL,
                    fields: vec![
                        ("Scheme".into(), 2),
                        ("Opaque".into(), 2),
                        ("Host".into(), 2),
                        ("Path".into(), 2),
                        ("RawPath".into(), 2),
                        ("ForceQuery".into(), 3),
                        ("RawQuery".into(), 2),
                        ("Fragment".into(), 2),
                        ("RawFragment".into(), 2),
                    ],
                },
                Instruction::BoxHeap {
                    dst: 13,
                    src: 16,
                    typ: TYPE_URL_PTR,
                },
                Instruction::CallMethod {
                    receiver: 16,
                    method: "String".into(),
                    args: vec![],
                    dst: Some(14),
                },
                Instruction::CallMethod {
                    receiver: 13,
                    method: "RequestURI".into(),
                    args: vec![],
                    dst: Some(15),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![14],
                    dst: None,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![15],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("program should run with direct opaque and force-query values");

    assert_eq!(
        vm.stdout(),
        "mailto:dev@example.com?\ndev@example.com?\ncustom://example.com/path?x=1\ncustom://example.com/path?x=1\n?\n/?\n"
    );
}
