use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    #[default]
    Uncategorized,
    CompileError,
    Tooling,
    ProtocolError,
    HostError,
    RuntimePanic,
    RuntimeTrap,
    RuntimeBudgetExhaustion,
    RuntimeDeadlock,
    RuntimeCancellation,
    RuntimeExit,
}
