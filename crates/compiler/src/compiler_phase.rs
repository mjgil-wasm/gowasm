use crate::CompileError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompilerPhase {
    ParseValidation,
    TypeChecking,
    Lowering,
    BytecodeEmission,
    RuntimeMetadataRegistration,
    #[allow(dead_code)]
    Diagnostics,
}

#[derive(Debug)]
pub(crate) struct PhaseFailure {
    #[allow(dead_code)]
    pub(crate) phase: CompilerPhase,
    pub(crate) error: CompileError,
    #[allow(dead_code)]
    pub(crate) emitted_function_count: usize,
    #[allow(dead_code)]
    pub(crate) emitted_debug_info_count: usize,
}

impl PhaseFailure {
    pub(crate) fn new(
        phase: CompilerPhase,
        error: CompileError,
        emitted_function_count: usize,
        emitted_debug_info_count: usize,
    ) -> Self {
        Self {
            phase,
            error,
            emitted_function_count,
            emitted_debug_info_count,
        }
    }

    pub(crate) fn into_compile_error(self) -> CompileError {
        self.error
    }
}
