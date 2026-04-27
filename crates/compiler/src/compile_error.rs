use gowasm_lexer::Span;
use gowasm_parser::ParseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error("package `{package}` must define a `main` function")]
    MissingMain { package: String },
    #[error("entry file `{path}` was not found in the source set")]
    MissingEntryFile { path: String },
    #[error("function `{name}` is defined more than once in package `{package}`")]
    DuplicateFunction { package: String, name: String },
    #[error("method `{type_name}.{name}` is defined more than once in package `{package}`")]
    DuplicateMethod {
        package: String,
        type_name: String,
        name: String,
    },
    #[error("type `{name}` is defined more than once in package `{package}`")]
    DuplicateType { package: String, name: String },
    #[error("package var `{name}` is defined more than once in package `{package}`")]
    DuplicateGlobal { package: String, name: String },
    #[error("method receiver type `{type_name}` is not a known named struct")]
    UnknownReceiverType { type_name: String },
    #[error("unsupported syntax: {detail}")]
    Unsupported { detail: String },
    #[error("function `{name}` is not defined in this file")]
    UnknownFunction { name: String },
    #[error("identifier `{name}` is not defined in the current function scope")]
    UnknownIdentifier { name: String },
    #[error("local `{name}` is duplicated in the current function scope")]
    DuplicateLocal { name: String },
    #[error("assignment target `{name}` is not defined in the current function scope")]
    UnknownAssignmentTarget { name: String },
    #[error("`break` can only be used inside a `for` loop or `switch`")]
    BreakOutsideBreakable,
    #[error("`continue` can only be used inside a `for` loop")]
    ContinueOutsideLoop,
    #[error("`{package}` must be imported before calling `{package}.{symbol}`")]
    MissingImport { package: String, symbol: String },
    #[error("import `{path}` could not be resolved while loading package `{importer}`")]
    UnresolvedImportPath { importer: String, path: String },
    #[error("invalid module root `{source_path}`: {detail}")]
    InvalidModuleRoot { source_path: String, detail: String },
    #[error(
        "unsupported module feature `{feature}` in `{source_path}`; the current subset only supports `module` and `go` directives"
    )]
    UnsupportedModuleFeature {
        source_path: String,
        feature: String,
    },
    #[error("module `{module_path}` is loaded from multiple cached versions: {versions:?}")]
    ConflictingModuleVersions {
        module_path: String,
        versions: Vec<String>,
    },
    #[error("import cycle detected: {cycle_path}")]
    ImportCycle {
        cycle: Vec<String>,
        cycle_path: String,
        source_path: String,
        span: Span,
    },
}
