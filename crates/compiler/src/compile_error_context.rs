use std::cell::RefCell;

use gowasm_lexer::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileErrorContext {
    pub file_path: String,
    pub span_start: usize,
    pub span_end: usize,
}

thread_local! {
    static LAST_COMPILE_ERROR_CONTEXT: RefCell<Option<CompileErrorContext>> = const { RefCell::new(None) };
}

pub(crate) fn clear_last_compile_error_context() {
    LAST_COMPILE_ERROR_CONTEXT.with(|context| {
        context.borrow_mut().take();
    });
}

pub fn take_last_compile_error_context() -> Option<CompileErrorContext> {
    LAST_COMPILE_ERROR_CONTEXT.with(|context| context.borrow_mut().take())
}

pub(crate) fn record_compile_error_context(path: impl Into<String>, span: Span) {
    LAST_COMPILE_ERROR_CONTEXT.with(|context| {
        *context.borrow_mut() = Some(CompileErrorContext {
            file_path: path.into(),
            span_start: span.start,
            span_end: span.end,
        });
    });
}
