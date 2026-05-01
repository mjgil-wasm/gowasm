use super::{CapabilityRequest, Function, Instruction, Program, RunOutcome, Value, Vm, VmError};

fn budget_program() -> Program {
    Program {
        entry_function: 0,
        methods: vec![],
        global_count: 1,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 1,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::StoreGlobal { global: 0, src: 0 },
                Instruction::Return { src: None },
            ],
        }],
    }
}

fn yielding_budget_program() -> Program {
    Program {
        entry_function: 0,
        methods: vec![],
        global_count: 1,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 1,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::StoreGlobal { global: 0, src: 0 },
                Instruction::LoadInt { dst: 0, value: 2 },
                Instruction::StoreGlobal { global: 0, src: 0 },
                Instruction::Return { src: None },
            ],
        }],
    }
}

fn callback_budget_program() -> Program {
    Program {
        entry_function: 0,
        methods: vec![],
        global_count: 1,
        functions: vec![
            Function {
                name: "main".into(),
                param_count: 0,
                register_count: 0,
                code: vec![Instruction::Return { src: None }],
            },
            Function {
                name: "callback".into(),
                param_count: 0,
                register_count: 1,
                code: vec![
                    Instruction::LoadInt { dst: 0, value: 7 },
                    Instruction::StoreGlobal { global: 0, src: 0 },
                    Instruction::Return { src: None },
                ],
            },
        ],
    }
}

#[test]
fn instruction_budget_stops_direct_execution_before_the_next_instruction() {
    let program = budget_program();
    let mut vm = Vm::new();
    vm.set_instruction_budget(2);

    let error = vm
        .run_program(&program)
        .expect_err("budget should stop the third instruction");
    assert!(matches!(
        error.root_cause(),
        VmError::InstructionBudgetExceeded {
            budget: 2,
            executed: 2,
            ..
        }
    ));
    assert_eq!(vm.executed_instruction_count(), 2);
    assert_eq!(vm.instruction_budget_remaining(), Some(0));
    assert_eq!(vm.globals[0], Value::int(1));
}

#[test]
fn instruction_budget_persists_across_resume_calls() {
    let program = yielding_budget_program();
    let mut vm = Vm::new();
    vm.enable_capability_requests();
    vm.set_instruction_yield_interval(1);
    vm.set_instruction_budget(2);

    assert_eq!(
        vm.start_program(&program)
            .expect("first instruction should yield"),
        RunOutcome::CapabilityRequest(CapabilityRequest::Yield)
    );
    assert_eq!(vm.executed_instruction_count(), 1);
    assert_eq!(vm.instruction_budget_remaining(), Some(1));

    assert_eq!(
        vm.resume_program(&program)
            .expect("second instruction should yield"),
        RunOutcome::CapabilityRequest(CapabilityRequest::Yield)
    );
    assert_eq!(vm.executed_instruction_count(), 2);
    assert_eq!(vm.instruction_budget_remaining(), Some(0));
    assert_eq!(vm.globals[0], Value::int(1));

    let error = vm
        .resume_program(&program)
        .expect_err("budget should stop execution before the third instruction");
    assert!(matches!(
        error.root_cause(),
        VmError::InstructionBudgetExceeded {
            budget: 2,
            executed: 2,
            ..
        }
    ));
    assert_eq!(vm.globals[0], Value::int(1));
}

#[test]
fn callback_execution_consumes_the_shared_instruction_budget() {
    let program = callback_budget_program();
    let mut vm = Vm::new();
    vm.set_instruction_budget(2);
    vm.reset_scheduler();
    vm.globals = vec![Value::nil(); program.global_count];
    vm.spawn_goroutine(&program, program.entry_function, Vec::new())
        .expect("main goroutine should spawn");

    let error = vm
        .invoke_callback_no_result(&program, 1, Vec::new())
        .expect_err("callback should exhaust the shared instruction budget");
    assert!(matches!(
        error,
        VmError::InstructionBudgetExceeded {
            budget: 2,
            executed: 2,
            ..
        }
    ));
    assert_eq!(vm.executed_instruction_count(), 2);
    assert_eq!(vm.instruction_budget_remaining(), Some(0));
    assert_eq!(vm.globals[0], Value::int(7));
}
