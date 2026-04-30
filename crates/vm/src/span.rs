use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::{Frame, Program, Vm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionSourceSpan {
    pub path: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionDebugInfo {
    pub instruction_spans: Vec<Option<InstructionSourceSpan>>,
}

impl FunctionDebugInfo {
    pub fn empty(len: usize) -> Self {
        Self {
            instruction_spans: vec![None; len],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFileDebugInfo {
    pub line_starts: Vec<usize>,
}

impl SourceFileDebugInfo {
    pub fn from_source(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (offset, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(offset + 1);
            }
        }
        Self { line_starts }
    }

    fn line_column(&self, offset: usize) -> (usize, usize) {
        let line_index = self
            .line_starts
            .partition_point(|line_start| *line_start <= offset);
        let line_index = line_index.saturating_sub(1);
        let line_start = self.line_starts.get(line_index).copied().unwrap_or(0);
        (line_index + 1, offset.saturating_sub(line_start) + 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameDebugInfo {
    pub function: String,
    pub instruction_index: usize,
    pub source_span: Option<InstructionSourceSpan>,
    pub source_location: Option<SourceLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramDebugInfo {
    pub functions: Vec<FunctionDebugInfo>,
    pub files: HashMap<String, SourceFileDebugInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ProgramDebugKey {
    functions_ptr: usize,
    function_count: usize,
}

fn registry() -> &'static Mutex<HashMap<ProgramDebugKey, Arc<ProgramDebugInfo>>> {
    static REGISTRY: OnceLock<Mutex<HashMap<ProgramDebugKey, Arc<ProgramDebugInfo>>>> =
        OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn debug_key(program: &Program) -> ProgramDebugKey {
    ProgramDebugKey {
        functions_ptr: program.functions.as_ptr() as usize,
        function_count: program.functions.len(),
    }
}

pub fn register_program_debug_info(program: &Program, debug_info: ProgramDebugInfo) {
    registry()
        .lock()
        .expect("program debug registry lock should not be poisoned")
        .insert(debug_key(program), Arc::new(debug_info));
}

pub fn program_debug_info(program: &Program) -> Option<ProgramDebugInfo> {
    lookup_program_debug_info(program).map(|debug_info| (*debug_info).clone())
}

pub(crate) fn lookup_program_debug_info(program: &Program) -> Option<Arc<ProgramDebugInfo>> {
    registry()
        .lock()
        .expect("program debug registry lock should not be poisoned")
        .get(&debug_key(program))
        .cloned()
}

impl Vm {
    pub fn current_frame_debug_info(&self, program: &Program) -> Option<FrameDebugInfo> {
        let frame = self.current_goroutine().frames.last()?;
        resolve_frame_debug_info(program, frame)
    }

    pub fn current_stack_debug_info(&self, program: &Program) -> Vec<FrameDebugInfo> {
        self.current_goroutine()
            .frames
            .iter()
            .rev()
            .filter_map(|frame| resolve_frame_debug_info(program, frame))
            .collect()
    }
}

fn resolve_frame_debug_info(program: &Program, frame: &Frame) -> Option<FrameDebugInfo> {
    let function = program.functions.get(frame.function)?;
    let instruction_index = resolve_instruction_index(function.code.len(), frame.pc);
    let debug_info = lookup_program_debug_info(program);
    let source_span = debug_info
        .as_ref()
        .and_then(|debug_info| debug_info.functions.get(frame.function))
        .and_then(|function_debug| function_debug.instruction_spans.get(instruction_index))
        .cloned()
        .flatten();
    let source_location = source_span.as_ref().and_then(|source_span| {
        debug_info
            .as_ref()
            .and_then(|debug_info| debug_info.files.get(&source_span.path))
            .map(|file_debug| {
                let (line, column) = file_debug.line_column(source_span.start);
                let end_offset = source_span.end.saturating_sub(1).max(source_span.start);
                let (end_line, end_column) = file_debug.line_column(end_offset);
                SourceLocation {
                    path: source_span.path.clone(),
                    line,
                    column,
                    end_line,
                    end_column,
                }
            })
    });
    Some(FrameDebugInfo {
        function: function.name.clone(),
        instruction_index,
        source_span,
        source_location,
    })
}

fn resolve_instruction_index(code_len: usize, pc: usize) -> usize {
    if code_len == 0 {
        return 0;
    }
    pc.saturating_sub(1).min(code_len - 1)
}
