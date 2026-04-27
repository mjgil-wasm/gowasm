use gowasm_host_types::ErrorCategory;

use super::{CapabilityRequest, TracedVmError, VmError};

#[test]
fn vm_error_categories_cover_runtime_failure_classes() {
    assert_eq!(
        VmError::UnhandledPanic {
            function: "main".into(),
            value: "boom".into(),
        }
        .category(),
        ErrorCategory::RuntimePanic
    );
    assert_eq!(
        VmError::DivisionByZero {
            function: "main".into(),
            left: "int value `1`".into(),
            right: "int value `0`".into(),
        }
        .category(),
        ErrorCategory::RuntimeTrap
    );
    assert_eq!(
        VmError::InstructionBudgetExceeded {
            function: "main".into(),
            budget: 50,
            executed: 50,
        }
        .category(),
        ErrorCategory::RuntimeBudgetExhaustion
    );
    assert_eq!(VmError::Deadlock.category(), ErrorCategory::RuntimeDeadlock);
    assert_eq!(
        VmError::ProgramExit { code: 2 }.category(),
        ErrorCategory::RuntimeExit
    );
    assert_eq!(
        VmError::CapabilityRequest {
            kind: CapabilityRequest::Yield,
        }
        .category(),
        ErrorCategory::ProtocolError
    );
}

#[test]
fn traced_vm_errors_use_their_root_cause_category() {
    let traced = VmError::Traced(Box::new(TracedVmError {
        root_cause: VmError::UnhandledPanic {
            function: "worker".into(),
            value: "boom".into(),
        },
        stack_trace: Vec::new(),
    }));

    assert_eq!(traced.category(), ErrorCategory::RuntimePanic);
}
