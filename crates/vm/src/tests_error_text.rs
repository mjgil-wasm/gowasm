use super::{Function, Instruction, Program, Vm};

#[test]
fn division_by_zero_reports_concrete_operands() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 3,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::LoadInt { dst: 1, value: 0 },
                Instruction::Divide {
                    dst: 2,
                    left: 0,
                    right: 1,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should fail");
    let text = error.to_string();
    assert!(text.contains("division by zero in function `main`"));
    assert!(text.contains("left int value `1`"));
    assert!(text.contains("right int value `0`"));
}

#[test]
fn invalid_function_values_report_the_target_shape() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 1,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::CallClosure {
                    callee: 0,
                    args: vec![],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should fail");
    let text = error.to_string();
    assert!(text.contains("call target must be a function value in function `main`"));
    assert!(text.contains("got int value `1`"));
}

#[test]
fn nil_map_assignment_reports_the_target_value() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 4,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "go".into(),
                },
                Instruction::LoadInt { dst: 1, value: 0 },
                Instruction::MakeNilMap {
                    dst: 2,
                    concrete_type: None,
                    zero: 1,
                },
                Instruction::LoadInt { dst: 3, value: 1 },
                Instruction::SetIndex {
                    target: 2,
                    index: 0,
                    src: 3,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should fail");
    let text = error.to_string();
    assert!(text.contains("cannot assign into a nil map in function `main`"));
    assert!(text.contains("target map value `map[]`"));
}

#[test]
fn send_on_closed_channel_reports_value_and_channel() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 3,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 0 },
                Instruction::MakeChannel {
                    concrete_type: None,
                    dst: 1,
                    cap: None,
                    zero: 0,
                },
                Instruction::CloseChannel { chan: 1 },
                Instruction::LoadInt { dst: 2, value: 7 },
                Instruction::ChanSend { chan: 1, value: 2 },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should fail");
    let text = error.to_string();
    assert!(text.contains("send on closed channel in function `main`"));
    assert!(text.contains("value int value `7`"));
    assert!(text.contains("target channel value `<chan>`"));
}
