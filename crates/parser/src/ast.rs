use gowasm_lexer::LexError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeParam {
    pub name: String,
    pub constraint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeConstraintRepr {
    Any,
    Comparable,
    Named(String),
    Interface(TypeConstraintInterfaceRepr),
}

impl TypeConstraintRepr {
    pub fn render(&self) -> String {
        match self {
            Self::Any => "interface{}".to_string(),
            Self::Comparable => "comparable".to_string(),
            Self::Named(name) => name.clone(),
            Self::Interface(interface) => interface.render(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeConstraintInterfaceRepr {
    pub methods: Vec<InterfaceMethodDecl>,
    pub embeds: Vec<String>,
    pub type_sets: Vec<Vec<String>>,
}

impl TypeConstraintInterfaceRepr {
    pub fn render(&self) -> String {
        let mut clauses = Vec::new();
        clauses.extend(self.methods.iter().map(render_interface_method));
        clauses.extend(self.embeds.iter().cloned());
        clauses.extend(self.type_sets.iter().map(|terms| terms.join("|")));
        format!("interface{{{}}}", clauses.join(";"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeChannelDirection {
    Bidirectional,
    SendOnly,
    ReceiveOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeRepr {
    Name(String),
    Pointer(Box<TypeRepr>),
    Slice(Box<TypeRepr>),
    Array {
        len: usize,
        element: Box<TypeRepr>,
    },
    Map {
        key: Box<TypeRepr>,
        value: Box<TypeRepr>,
    },
    Channel {
        direction: TypeChannelDirection,
        element: Box<TypeRepr>,
    },
    Function {
        params: Vec<TypeRepr>,
        results: Vec<TypeRepr>,
    },
    Struct {
        fields: Vec<TypeFieldDecl>,
    },
    Interface,
    GenericInstance {
        base: String,
        type_args: Vec<TypeRepr>,
    },
}

impl TypeRepr {
    pub fn render(&self) -> String {
        match self {
            Self::Name(name) => name.clone(),
            Self::Pointer(inner) => format!("*{}", inner.render()),
            Self::Slice(inner) => format!("[]{}", inner.render()),
            Self::Array { len, element } => format!("[{len}]{}", element.render()),
            Self::Map { key, value } => format!("map[{}]{}", key.render(), value.render()),
            Self::Channel { direction, element } => match direction {
                TypeChannelDirection::Bidirectional => format!("chan {}", element.render()),
                TypeChannelDirection::SendOnly => format!("chan<- {}", element.render()),
                TypeChannelDirection::ReceiveOnly => format!("<-chan {}", element.render()),
            },
            Self::Function { params, results } => format!(
                "__gowasm_func__({})->({})",
                params
                    .iter()
                    .map(TypeRepr::render)
                    .collect::<Vec<_>>()
                    .join(","),
                results
                    .iter()
                    .map(TypeRepr::render)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Self::Struct { fields } => format!(
                "struct{{{}}}",
                fields
                    .iter()
                    .map(render_struct_type_field)
                    .collect::<Vec<_>>()
                    .join(";")
            ),
            Self::Interface => "interface{}".to_string(),
            Self::GenericInstance { base, type_args } => format!(
                "{base}[{}]",
                type_args
                    .iter()
                    .map(TypeRepr::render)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub package_name: String,
    pub imports: Vec<ImportDecl>,
    pub types: Vec<TypeDecl>,
    pub consts: Vec<PackageConstDecl>,
    pub vars: Vec<PackageVarDecl>,
    pub functions: Vec<FunctionDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportDecl {
    pub alias: Option<String>,
    pub path: String,
}

impl ImportDecl {
    pub fn selector(&self) -> &str {
        self.alias
            .as_deref()
            .unwrap_or_else(|| self.path.rsplit('/').next().unwrap_or(&self.path))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDecl {
    pub receiver: Option<Parameter>,
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Parameter>,
    pub result_types: Vec<String>,
    pub result_names: Vec<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDecl {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub kind: TypeDeclKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeDeclKind {
    Struct {
        fields: Vec<TypeFieldDecl>,
    },
    Interface {
        methods: Vec<InterfaceMethodDecl>,
        embeds: Vec<String>,
    },
    Alias {
        underlying: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeFieldDecl {
    pub name: String,
    pub typ: String,
    pub embedded: bool,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterfaceMethodDecl {
    pub name: String,
    pub params: Vec<Parameter>,
    pub result_types: Vec<String>,
}

fn render_interface_method(method: &InterfaceMethodDecl) -> String {
    let params = method
        .params
        .iter()
        .map(render_parameter)
        .collect::<Vec<_>>()
        .join(",");
    match method.result_types.as_slice() {
        [] => format!("{}({params})", method.name),
        [result] => format!("{}({params}) {result}", method.name),
        results => format!("{}({params}) ({})", method.name, results.join(",")),
    }
}

fn render_parameter(parameter: &Parameter) -> String {
    let prefix = if parameter.variadic { "..." } else { "" };
    format!("{} {}{}", parameter.name, prefix, parameter.typ)
}

fn render_struct_type_field(field: &TypeFieldDecl) -> String {
    let mut rendered = if field.embedded {
        field.typ.clone()
    } else {
        format!("{} {}", field.name, field.typ)
    };
    if let Some(tag) = &field.tag {
        rendered.push(' ');
        rendered.push_str(tag);
    }
    rendered
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageVarDecl {
    pub name: String,
    pub typ: Option<String>,
    pub value: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageConstDecl {
    pub name: String,
    pub typ: Option<String>,
    pub value: Expr,
    pub iota: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstGroupDecl {
    pub name: String,
    pub typ: Option<String>,
    pub value: Expr,
    pub iota: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub typ: String,
    pub variadic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchCase {
    pub expressions: Vec<Expr>,
    pub body: Vec<Stmt>,
    pub fallthrough: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeSwitchCase {
    pub types: Vec<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectCase {
    pub stmt: Stmt,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Expr(Expr),
    VarDecl {
        name: String,
        typ: Option<String>,
        value: Option<Expr>,
    },
    ConstDecl {
        name: String,
        typ: Option<String>,
        value: Expr,
        iota: usize,
    },
    ConstGroup {
        decls: Vec<ConstGroupDecl>,
    },
    ShortVarDecl {
        name: String,
        value: Expr,
    },
    ShortVarDeclPair {
        first: String,
        second: String,
        value: Expr,
    },
    ShortVarDeclTriple {
        first: String,
        second: String,
        third: String,
        value: Expr,
    },
    ShortVarDeclQuad {
        first: String,
        second: String,
        third: String,
        fourth: String,
        value: Expr,
    },
    ShortVarDeclList {
        names: Vec<String>,
        values: Vec<Expr>,
    },
    Assign {
        target: AssignTarget,
        value: Expr,
    },
    AssignPair {
        first: AssignTarget,
        second: AssignTarget,
        value: Expr,
    },
    AssignTriple {
        first: AssignTarget,
        second: AssignTarget,
        third: AssignTarget,
        value: Expr,
    },
    AssignQuad {
        first: AssignTarget,
        second: AssignTarget,
        third: AssignTarget,
        fourth: AssignTarget,
        value: Expr,
    },
    AssignList {
        targets: Vec<AssignTarget>,
        values: Vec<Expr>,
    },
    Increment {
        name: String,
    },
    Decrement {
        name: String,
    },
    If {
        init: Option<Box<Stmt>>,
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },
    For {
        init: Option<Box<Stmt>>,
        condition: Option<Expr>,
        post: Option<Box<Stmt>>,
        body: Vec<Stmt>,
    },
    RangeFor {
        key: String,
        value: Option<String>,
        assign: bool,
        expr: Expr,
        body: Vec<Stmt>,
    },
    Switch {
        init: Option<Box<Stmt>>,
        expr: Option<Expr>,
        cases: Vec<SwitchCase>,
        default: Option<Vec<Stmt>>,
        default_index: Option<usize>,
        default_fallthrough: bool,
    },
    TypeSwitch {
        init: Option<Box<Stmt>>,
        binding: Option<String>,
        expr: Expr,
        cases: Vec<TypeSwitchCase>,
        default: Option<Vec<Stmt>>,
        default_index: Option<usize>,
    },
    Select {
        cases: Vec<SelectCase>,
        default: Option<Vec<Stmt>>,
    },
    Send {
        chan: Expr,
        value: Expr,
    },
    Go {
        call: Expr,
    },
    Defer {
        call: Expr,
    },
    Labeled {
        label: String,
        stmt: Box<Stmt>,
    },
    Break {
        label: Option<String>,
    },
    Continue {
        label: Option<String>,
    },
    Return(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignTarget {
    Ident(String),
    Deref { target: String },
    DerefSelector { target: String, field: String },
    DerefIndex { target: String, index: Expr },
    Selector { receiver: String, field: String },
    Index { target: String, index: Expr },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Ident(String),
    NilLiteral,
    BoolLiteral(bool),
    IntLiteral(i64),
    FloatLiteral(u64),
    StringLiteral(String),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    ArrayLiteral {
        len: usize,
        element_type: String,
        elements: Vec<Expr>,
    },
    SliceLiteral {
        element_type: String,
        elements: Vec<Expr>,
    },
    SliceConversion {
        element_type: String,
        expr: Box<Expr>,
    },
    MapLiteral {
        key_type: String,
        value_type: String,
        entries: Vec<MapLiteralEntry>,
    },
    StructLiteral {
        type_name: String,
        fields: Vec<StructLiteralField>,
    },
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
    },
    SliceExpr {
        target: Box<Expr>,
        low: Option<Box<Expr>>,
        high: Option<Box<Expr>>,
    },
    Selector {
        receiver: Box<Expr>,
        field: String,
    },
    TypeAssert {
        expr: Box<Expr>,
        asserted_type: String,
    },
    New {
        type_name: String,
    },
    Make {
        type_name: String,
        args: Vec<Expr>,
    },
    FunctionLiteral {
        params: Vec<Parameter>,
        result_types: Vec<String>,
        body: Vec<Stmt>,
    },
    Call {
        callee: Box<Expr>,
        type_args: Vec<String>,
        args: Vec<Expr>,
    },
    Spread {
        expr: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructLiteralField {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapLiteralEntry {
    pub key: Expr,
    pub value: Expr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    BitOr,
    BitXor,
    BitAnd,
    BitClear,
    Multiply,
    Divide,
    Modulo,
    ShiftLeft,
    ShiftRight,
    And,
    Or,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Negate,
    BitNot,
    AddressOf,
    Deref,
    Receive,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ParseError {
    #[error(transparent)]
    Lex(#[from] LexError),
    #[error("expected {expected}, found {found}")]
    UnexpectedToken { expected: String, found: String },
    #[error("unexpected end of input while parsing {context}")]
    UnexpectedEof { context: String },
}
