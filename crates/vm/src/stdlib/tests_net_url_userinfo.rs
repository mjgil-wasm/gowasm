use super::{
    resolve_stdlib_function, resolve_stdlib_method, resolve_stdlib_runtime_method,
    stdlib_function_param_types, stdlib_function_result_count, stdlib_function_result_types,
    stdlib_function_returns_value,
};
use crate::{
    Function, Instruction, Program, Vm, TYPE_URL, TYPE_URL_PTR, TYPE_URL_USERINFO,
    TYPE_URL_USERINFO_PTR,
};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should exist")
}

#[test]
fn resolves_net_url_userinfo_helpers_from_the_registry() {
    let user = resolve_stdlib_function("net/url", "User").expect("url.User should exist");
    assert!(stdlib_function_returns_value(user));
    assert_eq!(stdlib_function_result_count(user), 1);
    assert_eq!(stdlib_function_param_types(user), Some(&["string"][..]));
    assert_eq!(
        stdlib_function_result_types(user),
        Some(&["*url.Userinfo"][..])
    );

    let user_password =
        resolve_stdlib_function("net/url", "UserPassword").expect("url.UserPassword exists");
    assert!(stdlib_function_returns_value(user_password));
    assert_eq!(stdlib_function_result_count(user_password), 1);
    assert_eq!(
        stdlib_function_param_types(user_password),
        Some(&["string", "string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(user_password),
        Some(&["*url.Userinfo"][..])
    );

    let username = resolve_stdlib_method("*url.Userinfo", "Username")
        .expect("(*url.Userinfo).Username should exist");
    assert!(stdlib_function_returns_value(username));
    assert_eq!(stdlib_function_result_count(username), 1);
    assert_eq!(
        stdlib_function_param_types(username),
        Some(&["*url.Userinfo"][..])
    );
    assert_eq!(
        stdlib_function_result_types(username),
        Some(&["string"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_USERINFO_PTR, "Username"),
        Some(username)
    );

    let password = resolve_stdlib_method("*url.Userinfo", "Password")
        .expect("(*url.Userinfo).Password should exist");
    assert!(!stdlib_function_returns_value(password));
    assert_eq!(stdlib_function_result_count(password), 2);
    assert_eq!(
        stdlib_function_param_types(password),
        Some(&["*url.Userinfo"][..])
    );
    assert_eq!(
        stdlib_function_result_types(password),
        Some(&["string", "bool"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_URL_USERINFO_PTR, "Password"),
        Some(password)
    );
}

#[test]
fn net_url_runtime_userinfo_rendering_and_redaction_work() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 24,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "alice@example.com".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "p@ss:/?".into(),
                },
                Instruction::LoadBool {
                    dst: 2,
                    value: true,
                },
                Instruction::MakeStruct {
                    dst: 3,
                    typ: TYPE_URL_USERINFO,
                    fields: vec![
                        ("username".into(), 0),
                        ("password".into(), 1),
                        ("passwordSet".into(), 2),
                    ],
                },
                Instruction::BoxHeap {
                    dst: 4,
                    src: 3,
                    typ: TYPE_URL_USERINFO_PTR,
                },
                Instruction::LoadString {
                    dst: 5,
                    value: "https".into(),
                },
                Instruction::LoadString {
                    dst: 6,
                    value: String::new(),
                },
                Instruction::LoadString {
                    dst: 7,
                    value: "example.com".into(),
                },
                Instruction::LoadString {
                    dst: 8,
                    value: "/path".into(),
                },
                Instruction::LoadString {
                    dst: 9,
                    value: String::new(),
                },
                Instruction::LoadBool {
                    dst: 10,
                    value: false,
                },
                Instruction::MakeStruct {
                    dst: 11,
                    typ: TYPE_URL,
                    fields: vec![
                        ("Scheme".into(), 5),
                        ("Opaque".into(), 6),
                        ("User".into(), 4),
                        ("Host".into(), 7),
                        ("Path".into(), 8),
                        ("RawPath".into(), 9),
                        ("ForceQuery".into(), 10),
                        ("RawQuery".into(), 9),
                        ("Fragment".into(), 9),
                        ("RawFragment".into(), 9),
                    ],
                },
                Instruction::BoxHeap {
                    dst: 12,
                    src: 11,
                    typ: TYPE_URL_PTR,
                },
                Instruction::CallMethod {
                    receiver: 4,
                    method: "String".into(),
                    args: vec![],
                    dst: Some(13),
                },
                Instruction::CallMethod {
                    receiver: 4,
                    method: "Username".into(),
                    args: vec![],
                    dst: Some(14),
                },
                Instruction::CallMethodMulti {
                    receiver: 4,
                    method: "Password".into(),
                    args: vec![],
                    dsts: vec![15, 16],
                },
                Instruction::CallMethod {
                    receiver: 11,
                    method: "String".into(),
                    args: vec![],
                    dst: Some(17),
                },
                Instruction::CallMethod {
                    receiver: 12,
                    method: "Redacted".into(),
                    args: vec![],
                    dst: Some(18),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![13],
                    dst: None,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![14],
                    dst: None,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![15, 16],
                    dst: None,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![17],
                    dst: None,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![18],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program)
        .expect("program should run with runtime userinfo values");
    assert_eq!(
        vm.stdout(),
        "alice%40example.com:p%40ss%3A%2F%3F\nalice@example.com\np@ss:/? true\nhttps://alice%40example.com:p%40ss%3A%2F%3F@example.com/path\nhttps://alice%40example.com:xxxxx@example.com/path\n"
    );
}
