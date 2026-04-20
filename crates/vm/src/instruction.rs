use super::{ConcreteType, StdlibFunctionId, TypeCheck, TypeId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareOp {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectCaseOp {
    pub chan: usize,
    pub kind: SelectCaseOpKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectCaseOpKind {
    Recv {
        value_dst: usize,
        ok_dst: Option<usize>,
    },
    Send {
        value: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Instruction {
    LoadInt {
        dst: usize,
        value: i64,
    },
    LoadBool {
        dst: usize,
        value: bool,
    },
    LoadFloat {
        dst: usize,
        value: super::Float64,
    },
    LoadString {
        dst: usize,
        value: String,
    },
    LoadNil {
        dst: usize,
    },
    LoadNilChannel {
        dst: usize,
        concrete_type: Option<ConcreteType>,
    },
    LoadNilPointer {
        dst: usize,
        typ: TypeId,
        concrete_type: Option<ConcreteType>,
    },
    BoxHeap {
        dst: usize,
        src: usize,
        typ: TypeId,
    },
    AddressLocal {
        dst: usize,
        src: usize,
        typ: TypeId,
    },
    AddressGlobal {
        dst: usize,
        global: usize,
        typ: TypeId,
    },
    ProjectFieldPointer {
        dst: usize,
        src: usize,
        field: String,
        typ: TypeId,
    },
    ProjectIndexPointer {
        dst: usize,
        src: usize,
        index: usize,
        typ: TypeId,
    },
    AddressLocalField {
        dst: usize,
        src: usize,
        field: String,
        typ: TypeId,
    },
    AddressGlobalField {
        dst: usize,
        global: usize,
        field: String,
        typ: TypeId,
    },
    AddressLocalIndex {
        dst: usize,
        src: usize,
        index: usize,
        typ: TypeId,
    },
    AddressGlobalIndex {
        dst: usize,
        global: usize,
        index: usize,
        typ: TypeId,
    },
    LoadNilSlice {
        dst: usize,
        concrete_type: Option<ConcreteType>,
    },
    LoadErrorMessage {
        dst: usize,
        src: usize,
    },
    LoadGlobal {
        dst: usize,
        global: usize,
    },
    StoreGlobal {
        global: usize,
        src: usize,
    },
    MakeArray {
        dst: usize,
        concrete_type: Option<ConcreteType>,
        items: Vec<usize>,
    },
    MakeSlice {
        dst: usize,
        concrete_type: Option<ConcreteType>,
        items: Vec<usize>,
    },
    MakeChannel {
        dst: usize,
        concrete_type: Option<ConcreteType>,
        cap: Option<usize>,
        zero: usize,
    },
    MakeMap {
        dst: usize,
        concrete_type: Option<ConcreteType>,
        entries: Vec<(usize, usize)>,
        zero: usize,
    },
    MakeNilMap {
        dst: usize,
        concrete_type: Option<ConcreteType>,
        zero: usize,
    },
    MakeStruct {
        dst: usize,
        typ: TypeId,
        fields: Vec<(String, usize)>,
    },
    Index {
        dst: usize,
        target: usize,
        index: usize,
    },
    Slice {
        dst: usize,
        target: usize,
        low: Option<usize>,
        high: Option<usize>,
    },
    MapContains {
        dst: usize,
        target: usize,
        index: usize,
    },
    GetField {
        dst: usize,
        target: usize,
        field: String,
    },
    AssertType {
        dst: usize,
        src: usize,
        target: TypeCheck,
    },
    TypeMatches {
        dst: usize,
        src: usize,
        target: TypeCheck,
    },
    IsNil {
        dst: usize,
        src: usize,
    },
    SetField {
        target: usize,
        field: String,
        src: usize,
    },
    SetIndex {
        target: usize,
        index: usize,
        src: usize,
    },
    StoreIndirect {
        target: usize,
        src: usize,
    },
    Copy {
        target: usize,
        src: usize,
        count_dst: Option<usize>,
    },
    Move {
        dst: usize,
        src: usize,
    },
    Deref {
        dst: usize,
        src: usize,
    },
    Not {
        dst: usize,
        src: usize,
    },
    Negate {
        dst: usize,
        src: usize,
    },
    BitNot {
        dst: usize,
        src: usize,
    },
    Add {
        dst: usize,
        left: usize,
        right: usize,
    },
    Subtract {
        dst: usize,
        left: usize,
        right: usize,
    },
    BitXor {
        dst: usize,
        left: usize,
        right: usize,
    },
    BitAnd {
        dst: usize,
        left: usize,
        right: usize,
    },
    BitClear {
        dst: usize,
        left: usize,
        right: usize,
    },
    BitOr {
        dst: usize,
        left: usize,
        right: usize,
    },
    Multiply {
        dst: usize,
        left: usize,
        right: usize,
    },
    Divide {
        dst: usize,
        left: usize,
        right: usize,
    },
    Modulo {
        dst: usize,
        left: usize,
        right: usize,
    },
    ShiftLeft {
        dst: usize,
        left: usize,
        right: usize,
    },
    ShiftRight {
        dst: usize,
        left: usize,
        right: usize,
    },
    Compare {
        dst: usize,
        op: CompareOp,
        left: usize,
        right: usize,
    },
    Jump {
        target: usize,
    },
    JumpIfFalse {
        cond: usize,
        target: usize,
    },
    Select {
        choice_dst: usize,
        cases: Vec<SelectCaseOp>,
        default_case: Option<usize>,
    },
    GoCall {
        function: usize,
        args: Vec<usize>,
    },
    GoCallClosure {
        callee: usize,
        args: Vec<usize>,
    },
    GoCallMethod {
        receiver: usize,
        method: String,
        args: Vec<usize>,
    },
    ChanSend {
        chan: usize,
        value: usize,
    },
    ChanRecv {
        dst: usize,
        chan: usize,
    },
    ChanRecvOk {
        value_dst: usize,
        ok_dst: usize,
        chan: usize,
    },
    ChanTryRecv {
        ready_dst: usize,
        value_dst: usize,
        chan: usize,
    },
    ChanTryRecvOk {
        ready_dst: usize,
        value_dst: usize,
        ok_dst: usize,
        chan: usize,
    },
    ChanTrySend {
        ready_dst: usize,
        chan: usize,
        value: usize,
    },
    CloseChannel {
        chan: usize,
    },
    CallStdlib {
        function: StdlibFunctionId,
        args: Vec<usize>,
        dst: Option<usize>,
    },
    DeferStdlib {
        function: StdlibFunctionId,
        args: Vec<usize>,
    },
    GoCallStdlib {
        function: StdlibFunctionId,
        args: Vec<usize>,
    },
    CallStdlibMulti {
        function: StdlibFunctionId,
        args: Vec<usize>,
        dsts: Vec<usize>,
    },
    CallFunction {
        function: usize,
        args: Vec<usize>,
        dst: Option<usize>,
    },
    MakeClosure {
        dst: usize,
        concrete_type: Option<ConcreteType>,
        function: usize,
        captures: Vec<usize>,
    },
    CallClosure {
        callee: usize,
        args: Vec<usize>,
        dst: Option<usize>,
    },
    DeferClosure {
        callee: usize,
        args: Vec<usize>,
    },
    DeferFunction {
        function: usize,
        args: Vec<usize>,
    },
    CallFunctionMulti {
        function: usize,
        args: Vec<usize>,
        dsts: Vec<usize>,
    },
    CallClosureMulti {
        callee: usize,
        args: Vec<usize>,
        dsts: Vec<usize>,
    },
    CallMethod {
        receiver: usize,
        method: String,
        args: Vec<usize>,
        dst: Option<usize>,
    },
    DeferMethod {
        receiver: usize,
        method: String,
        args: Vec<usize>,
    },
    CallMethodMulti {
        receiver: usize,
        method: String,
        args: Vec<usize>,
        dsts: Vec<usize>,
    },
    CallMethodMultiMutatingArg {
        receiver: usize,
        method: String,
        args: Vec<usize>,
        dsts: Vec<usize>,
        mutated_arg: usize,
    },
    Return {
        src: Option<usize>,
    },
    ReturnMulti {
        srcs: Vec<usize>,
    },
    Panic {
        src: usize,
    },
    Recover {
        dst: usize,
    },
    ConvertToInt {
        dst: usize,
        src: usize,
    },
    ConvertToFloat64 {
        dst: usize,
        src: usize,
    },
    ConvertToString {
        dst: usize,
        src: usize,
    },
    ConvertToByte {
        dst: usize,
        src: usize,
    },
    ConvertToByteSlice {
        dst: usize,
        src: usize,
    },
    ConvertToRuneSlice {
        dst: usize,
        src: usize,
    },
    ConvertRuneSliceToString {
        dst: usize,
        src: usize,
    },
    Retag {
        dst: usize,
        src: usize,
        typ: TypeId,
    },
}
