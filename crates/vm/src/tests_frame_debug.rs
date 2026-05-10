use std::collections::HashMap;

use crate::{
    register_program_debug_info, CapabilityRequest, Function, FunctionDebugInfo, Instruction,
    InstructionSourceSpan, Program, ProgramDebugInfo, RunOutcome, SourceFileDebugInfo, Vm,
};

fn debug_program(source: &str) -> Program {
    let program = Program {
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 0,
                code: vec![
                    Instruction::CallFunction {
                        function: 1,
                        args: vec![],
                        dst: None,
                    },
                    Instruction::Return { src: None },
                ],
            },
            Function {
                name: "helper".into(),
                param_count: 0,
                register_count: 2,
                code: vec![
                    Instruction::LoadInt { dst: 0, value: 1 },
                    Instruction::LoadInt { dst: 1, value: 2 },
                    Instruction::Return { src: None },
                ],
            },
        ],
        methods: Vec::new(),
        global_count: 0,
        entry_function: 0,
    };

    let helper_call_start = source
        .rfind("helper()")
        .expect("source should contain helper call");
    let second_stmt_start = source
        .find("b := 2")
        .expect("source should contain b statement");
    register_program_debug_info(
        &program,
        ProgramDebugInfo {
            functions: vec![
                FunctionDebugInfo {
                    instruction_spans: vec![
                        Some(InstructionSourceSpan {
                            path: "main.go".into(),
                            start: helper_call_start,
                            end: helper_call_start + "helper()".len(),
                        }),
                        None,
                    ],
                },
                FunctionDebugInfo {
                    instruction_spans: vec![
                        None,
                        Some(InstructionSourceSpan {
                            path: "main.go".into(),
                            start: second_stmt_start,
                            end: second_stmt_start + "b := 2".len(),
                        }),
                        None,
                    ],
                },
            ],
            files: HashMap::from([("main.go".into(), SourceFileDebugInfo::from_source(source))]),
        },
    );
    program
}

#[test]
fn current_frame_debug_info_maps_to_the_active_source_location() {
    let source = "package main\n\nfunc helper() {\n    a := 1\n    b := 2\n}\n\nfunc main() {\n    helper()\n}\n";
    let program = debug_program(source);
    let mut vm = Vm::new();
    vm.enable_capability_requests();
    vm.set_instruction_yield_interval(3);

    let outcome = vm.start_program(&program).expect("program should start");
    assert_eq!(
        outcome,
        RunOutcome::CapabilityRequest(CapabilityRequest::Yield)
    );

    let frame = vm
        .current_frame_debug_info(&program)
        .expect("current frame debug info should exist");
    assert_eq!(frame.function, "helper");
    assert_eq!(frame.instruction_index, 1);
    let source_span = frame
        .source_span
        .expect("current frame should resolve a source span");
    assert_eq!(source_span.path, "main.go");
    assert_eq!(&source[source_span.start..source_span.end], "b := 2");
    let source_location = frame
        .source_location
        .expect("current frame should resolve a source location");
    assert_eq!(source_location.path, "main.go");
    assert_eq!(source_location.line, 5);
    assert_eq!(source_location.column, 5);
}

#[test]
fn current_stack_debug_info_maps_caller_and_callee_frames() {
    let source = "package main\n\nfunc helper() {\n    a := 1\n    b := 2\n}\n\nfunc main() {\n    helper()\n}\n";
    let program = debug_program(source);
    let mut vm = Vm::new();
    vm.enable_capability_requests();
    vm.set_instruction_yield_interval(3);

    let outcome = vm.start_program(&program).expect("program should start");
    assert_eq!(
        outcome,
        RunOutcome::CapabilityRequest(CapabilityRequest::Yield)
    );

    let stack = vm.current_stack_debug_info(&program);
    assert_eq!(stack.len(), 2);
    assert_eq!(stack[0].function, "helper");
    assert_eq!(stack[0].instruction_index, 1);
    assert_eq!(
        stack[0]
            .source_location
            .as_ref()
            .expect("callee frame should resolve a source location")
            .line,
        5
    );
    assert_eq!(stack[1].function, "main");
    assert_eq!(stack[1].instruction_index, 0);
    let main_location = stack[1]
        .source_location
        .as_ref()
        .expect("caller frame should resolve a source location");
    assert_eq!(main_location.line, 9);
    assert_eq!(main_location.column, 5);
}
