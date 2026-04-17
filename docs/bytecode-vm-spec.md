# Bytecode And VM Spec

This document defines the checked internal bytecode surface used by
`gowasm-vm`.

The VM does not ingest arbitrary external bytecode blobs. The instruction
surface is the internal `Instruction` enum in `crates/vm/src/instruction.rs`,
and this spec is the checked description maintainers must keep aligned with
that enum.

## Register Convention

- every compiled function owns a zero-based register file sized by
  `Function.register_count`
- instruction operands named `dst`, `src`, `left`, `right`, `target`,
  `receiver`, `callee`, `value`, `cond`, `choice_dst`, `value_dst`, `ok_dst`,
  `ready_dst`, `mutated_arg`, and `dsts` all refer to registers in the current
  frame unless the operand name explicitly says `global`, `function`, or
  `chan`
- `global` operands index the program-global slot array
- `function` operands index `Program.functions`
- `StdlibFunctionId` operands identify registered stdlib handlers
- `TypeId`, `ConcreteType`, `TypeCheck`, `CompareOp`, and `SelectCaseOp`
  operands carry runtime type or control metadata rather than register indexes
- `Vec<usize>` operands preserve source order; the compiler is responsible for
  arranging argument and capture lists in call or literal order

## Frame Behavior

- a `Program` enters through `Program.entry_function`
- each frame stores the current function index, program counter, register file,
  deferred-call stack, unwind state, and return target
- direct calls, closure calls, method calls, goroutine launches, and stdlib
  calls either push a new frame or route through the stdlib dispatch table
- `Return` and `ReturnMulti` complete the current frame and write results back
  through the recorded return target
- `Defer*` instructions append work to the current frame's deferred stack in
  source order; unwind drains that stack in LIFO order
- `Panic` moves the frame into unwind, `Recover` can clear the active panic
  only from an eligible deferred frame, and an unhandled panic leaves the VM as
  a traced runtime failure

## Trap Behavior

- invalid operand-type combinations, divide-by-zero, bad conversions, failed
  type assertions, nil misuse, closed-channel misuse, out-of-bounds access, and
  other runtime faults surface as `VmError` values that the engine later wraps
  into traced diagnostics
- channel, select, and host-wait instructions may pause execution and yield a
  capability request instead of completing synchronously
- instruction-budget exhaustion is treated like an ordinary traced runtime
  failure, not silent cancellation

## Source-Span Expectation

- instructions are emitted into per-function code buffers
- compiler-side source spans are attached separately through
  `InstructionSourceSpan` entries in `ProgramDebugInfo`; the instruction enum
  itself does not carry span fields
- maintainers should preserve a one-instruction-to-one-span-slot expectation:
  if code length changes, the debug info for that function must stay aligned by
  instruction index

## Operand Types

- `usize`: register, function, global, or channel slot index depending on field
  name
- `Option<usize>`: optional register or optional branch target
- `Vec<usize>`: ordered register lists
- `TypeId`: runtime-visible type identity token
- `ConcreteType`: concrete runtime shape metadata used for nil values,
  collections, closures, and reflection-sensitive paths
- `TypeCheck`: runtime assertion or type-switch target descriptor
- `CompareOp`: comparison opcode selector
- `SelectCaseOp`: lowered select-case operation descriptor
- `StdlibFunctionId`: registered stdlib call target

## Instruction Catalog

### Load And Address Instructions

| Instruction | Operands | Purpose |
| --- | --- | --- |
| `LoadInt` | `dst`, `value` | Write an integer literal into a register. |
| `LoadBool` | `dst`, `value` | Write a boolean literal into a register. |
| `LoadFloat` | `dst`, `value` | Write a float literal into a register. |
| `LoadString` | `dst`, `value` | Write a string literal into a register. |
| `LoadNil` | `dst` | Write the untyped nil value into a register. |
| `LoadNilChannel` | `dst`, `concrete_type` | Write a typed nil channel value. |
| `LoadNilPointer` | `dst`, `typ`, `concrete_type` | Write a typed nil pointer value. |
| `BoxHeap` | `dst`, `src`, `typ` | Move a value onto heap-backed storage and return a pointer wrapper. |
| `AddressLocal` | `dst`, `src`, `typ` | Form a pointer to a local register slot. |
| `AddressGlobal` | `dst`, `global`, `typ` | Form a pointer to a global slot. |
| `ProjectFieldPointer` | `dst`, `src`, `field`, `typ` | Project a field pointer from an existing pointer value. |
| `ProjectIndexPointer` | `dst`, `src`, `index`, `typ` | Project an index pointer from an existing pointer value. |
| `AddressLocalField` | `dst`, `src`, `field`, `typ` | Form a pointer to a field on a local aggregate. |
| `AddressGlobalField` | `dst`, `global`, `field`, `typ` | Form a pointer to a field on a global aggregate. |
| `AddressLocalIndex` | `dst`, `src`, `index`, `typ` | Form a pointer to an indexed local aggregate element. |
| `AddressGlobalIndex` | `dst`, `global`, `index`, `typ` | Form a pointer to an indexed global aggregate element. |
| `LoadNilSlice` | `dst`, `concrete_type` | Write a typed nil slice value. |
| `LoadErrorMessage` | `dst`, `src` | Extract an error's message string. |
| `LoadGlobal` | `dst`, `global` | Read a global slot into a register. |
| `StoreGlobal` | `global`, `src` | Write a register into a global slot. |

### Aggregate And Access Instructions

| Instruction | Operands | Purpose |
| --- | --- | --- |
| `MakeArray` | `dst`, `concrete_type`, `items` | Construct an array value from registers. |
| `MakeSlice` | `dst`, `concrete_type`, `items` | Construct a slice value from registers. |
| `MakeChannel` | `dst`, `concrete_type`, `cap`, `zero` | Construct a typed channel value. |
| `MakeMap` | `dst`, `concrete_type`, `entries`, `zero` | Construct a map value from key/value register pairs. |
| `MakeNilMap` | `dst`, `concrete_type`, `zero` | Construct a typed nil map wrapper. |
| `MakeStruct` | `dst`, `typ`, `fields` | Construct a struct value from named field/register pairs. |
| `Index` | `dst`, `target`, `index` | Read an indexed value from an array, slice, or string-like source. |
| `Slice` | `dst`, `target`, `low`, `high` | Slice a string, array, or slice target. |
| `MapContains` | `dst`, `target`, `index` | Report whether a map contains a key. |
| `GetField` | `dst`, `target`, `field` | Read a named field from a struct-like value. |
| `AssertType` | `dst`, `src`, `target` | Perform a runtime type assertion that traps on failure. |
| `TypeMatches` | `dst`, `src`, `target` | Report whether a runtime value matches a type-check target. |
| `IsNil` | `dst`, `src` | Report nil-ness for nilable runtime values. |
| `SetField` | `target`, `field`, `src` | Mutate a field on an aggregate value in place. |
| `SetIndex` | `target`, `index`, `src` | Mutate an indexed aggregate element in place. |
| `StoreIndirect` | `target`, `src` | Write through a pointer target. |
| `Copy` | `target`, `src`, `count_dst` | Copy between slices and optionally write the copied count. |
| `Move` | `dst`, `src` | Copy a register value into another register. |
| `Deref` | `dst`, `src` | Read through a pointer value. |

### Unary, Arithmetic, And Comparison Instructions

| Instruction | Operands | Purpose |
| --- | --- | --- |
| `Not` | `dst`, `src` | Boolean logical negation. |
| `Negate` | `dst`, `src` | Numeric unary negation. |
| `BitNot` | `dst`, `src` | Integer bitwise complement. |
| `Add` | `dst`, `left`, `right` | Addition or concatenation over the supported value families. |
| `Subtract` | `dst`, `left`, `right` | Numeric subtraction. |
| `BitXor` | `dst`, `left`, `right` | Integer bitwise XOR. |
| `BitAnd` | `dst`, `left`, `right` | Integer bitwise AND. |
| `BitClear` | `dst`, `left`, `right` | Integer bit clear (`&^`). |
| `BitOr` | `dst`, `left`, `right` | Integer bitwise OR. |
| `Multiply` | `dst`, `left`, `right` | Numeric multiplication. |
| `Divide` | `dst`, `left`, `right` | Numeric division; traps on divide by zero. |
| `Modulo` | `dst`, `left`, `right` | Integer remainder; traps on divide by zero. |
| `ShiftLeft` | `dst`, `left`, `right` | Integer left shift. |
| `ShiftRight` | `dst`, `left`, `right` | Integer right shift. |
| `Compare` | `dst`, `op`, `left`, `right` | Apply a `CompareOp` to two operands. |

### Control-Flow And Concurrency Instructions

| Instruction | Operands | Purpose |
| --- | --- | --- |
| `Jump` | `target` | Unconditional branch to an instruction index. |
| `JumpIfFalse` | `cond`, `target` | Branch when a condition register is false. |
| `Select` | `choice_dst`, `cases`, `default_case` | Lowered select operation over channel cases. |
| `GoCall` | `function`, `args` | Launch a goroutine for a direct function target. |
| `GoCallClosure` | `callee`, `args` | Launch a goroutine for a closure value. |
| `GoCallMethod` | `receiver`, `method`, `args` | Launch a goroutine for a method call target. |
| `ChanSend` | `chan`, `value` | Send on a channel, blocking when required. |
| `ChanRecv` | `dst`, `chan` | Receive from a channel into one register. |
| `ChanRecvOk` | `value_dst`, `ok_dst`, `chan` | Receive from a channel and also report comma-ok state. |
| `ChanTryRecv` | `ready_dst`, `value_dst`, `chan` | Non-blocking receive with readiness bit. |
| `ChanTryRecvOk` | `ready_dst`, `value_dst`, `ok_dst`, `chan` | Non-blocking receive with readiness and comma-ok bits. |
| `ChanTrySend` | `ready_dst`, `chan`, `value` | Non-blocking send with readiness bit. |
| `CloseChannel` | `chan` | Close a channel; traps on invalid close paths. |

### Call, Closure, And Defer Instructions

| Instruction | Operands | Purpose |
| --- | --- | --- |
| `CallStdlib` | `function`, `args`, `dst` | Call a stdlib function that returns zero or one result. |
| `DeferStdlib` | `function`, `args` | Defer a stdlib call on the current frame. |
| `GoCallStdlib` | `function`, `args` | Launch a goroutine for a stdlib function call. |
| `CallStdlibMulti` | `function`, `args`, `dsts` | Call a stdlib function with multiple results. |
| `CallFunction` | `function`, `args`, `dst` | Call a direct function target. |
| `MakeClosure` | `dst`, `concrete_type`, `function`, `captures` | Construct a closure value with captured registers. |
| `CallClosure` | `callee`, `args`, `dst` | Call a closure value. |
| `DeferClosure` | `callee`, `args` | Defer a closure call. |
| `DeferFunction` | `function`, `args` | Defer a direct function call. |
| `CallFunctionMulti` | `function`, `args`, `dsts` | Call a direct function target with multiple results. |
| `CallClosureMulti` | `callee`, `args`, `dsts` | Call a closure value with multiple results. |
| `CallMethod` | `receiver`, `method`, `args`, `dst` | Call a method on a receiver value. |
| `DeferMethod` | `receiver`, `method`, `args` | Defer a method call. |
| `CallMethodMulti` | `receiver`, `method`, `args`, `dsts` | Call a method that returns multiple values. |
| `CallMethodMultiMutatingArg` | `receiver`, `method`, `args`, `dsts`, `mutated_arg` | Call a method that both returns multiple values and mutates a caller-visible argument. |

### Return, Panic, And Conversion Instructions

| Instruction | Operands | Purpose |
| --- | --- | --- |
| `Return` | `src` | Return zero or one value from the current frame. |
| `ReturnMulti` | `srcs` | Return multiple values from the current frame. |
| `Panic` | `src` | Raise a panic from a value register. |
| `Recover` | `dst` | Recover the active panic when the frame is eligible. |
| `ConvertToInt` | `dst`, `src` | Convert a runtime value to `int`. |
| `ConvertToFloat64` | `dst`, `src` | Convert a runtime value to `float64`. |
| `ConvertToString` | `dst`, `src` | Convert a runtime value to `string`. |
| `ConvertToByte` | `dst`, `src` | Convert a runtime value to `byte`. |
| `ConvertToByteSlice` | `dst`, `src` | Convert a runtime value to `[]byte`. |
| `ConvertToRuneSlice` | `dst`, `src` | Convert a runtime value to `[]rune`. |
| `ConvertRuneSliceToString` | `dst`, `src` | Convert a runtime `[]rune` value to `string`. |
| `Retag` | `dst`, `src`, `typ` | Rewrap a runtime value with a more specific `TypeId`. |
