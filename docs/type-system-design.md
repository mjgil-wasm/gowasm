# Type System Design

This document describes the current internal type-system design used by
`gowasm` and the boundaries maintainers should preserve while the broader
compiler type refactor remains open.

## Layers

The current implementation has three distinct type layers:

1. Parser surface syntax
   The parser now first lowers every parsed type into a canonical `TypeRepr`
   tree covering names, pointers, slices, arrays, maps, channels, functions,
   interfaces, and instantiated generics. The legacy string fields in the AST
   are rendered from that canonical tree so existing frontend/compiler users
   can migrate incrementally without reintroducing ad hoc type parsing.
2. Compiler semantic normalization
   The compiler resolves those parser-rendered spellings into named-type
   tables, pointer-type tables, generic instantiations, assignability checks,
   call validation, and runtime type inventory entries. Generic substitution
   and inference now walk a canonical internal `TypeKey` tree instead of
   manually splitting nested strings.
3. Runtime type identity
   The VM and reflection-sensitive stdlib paths use `TypeId`, `ConcreteType`,
   and `RuntimeTypeInfo` to preserve runtime-visible shape identity.

Task `013` is now complete: function-signature assignability now lowers
compiler-canonical `__gowasm_func__` spellings into structural signature keys
before comparing params/results, while the legacy rendered strings remain only
as compatibility output for downstream callers that still consume text.

## Canonical Type Representation

The current canonical runtime-facing representation is:

- `TypeRepr`
  Parser-side canonical type tree used before AST compatibility rendering.
- `TypeKey`
  Compiler-side canonical type tree used for structural generic substitution
  and inference over nested arrays, slices, maps, channels, functions,
  pointers, interfaces, and instantiated generics.
- `TypeId`
  Stable runtime identity token for named types and selected built-in/runtime
  shapes.
- `ConcreteType`
  Structural runtime shape descriptor for arrays, slices, maps, pointers,
  functions, channels, and direct `TypeId` references.
- `RuntimeTypeInfo`
  Reflection- and JSON-facing metadata record keyed by `TypeId`, including
  display name, package path, kind, fields, element/key relationships, function
  params/results, underlying alias shape, and channel direction.

The parser and compiler normalize source spellings into those forms when they
need stable identity for lowering, substitution, reflection, JSON, or imported
package artifacts.

## Named And Unnamed Identity

- named structs, interfaces, aliases, and instantiated generic named types get
  stable `TypeId` values
- unnamed collection/function/channel/pointer shapes can still be represented
  canonically through `ConcreteType`
- named identities win over wrapper metadata when both are available; runtime
  lookup prefers the explicit `TypeId` before falling back to a structural
  wrapper

This is why named aliases like `Labels` and instantiated generic named types
like `Box[int]` preserve their names through runtime inventory, typed runtime
lookup, function signatures, and named generic metadata instead of collapsing
to an anonymous struct shape. Some direct `reflect.TypeOf` queries over alias
slice values still expose the underlying structural spelling (`[]string`), so
the named inventory entry remains the authoritative identity record for those
paths until task `012` lands.

## Aliases

- alias declarations keep their source-level name and package path
- alias runtime metadata records the visible alias kind plus an `underlying`
  `ConcreteType`
- reflect- and JSON-sensitive code can therefore distinguish a named alias from
  its unnamed underlying shape when the runtime value carries that alias
  `TypeId`

## Generics

- generic declarations remain template definitions until concrete use forces
  instantiation
- an instantiation cache keyed by concrete type arguments ensures equal generic
  instances reuse one compiled function or type identity within a compilation
  pass
- concrete named generic instances such as `Box[int]` receive their own
  runtime-visible `TypeId`, method set, and `RuntimeTypeInfo`

## Interfaces

- named interfaces receive their own `TypeId`
- interface satisfaction is checked against method sets during compiler
  validation and reflected at runtime through the current `TypeCheck` and
  method-binding model
- typed nil interfaces preserve interface identity through runtime type lookup
  instead of collapsing to untyped nil

## Function Types

- source-level `func(...) ...` spellings are normalized into canonical function
  parameter and result lists during compiler validation
- compiler-side function assignability now compares structural signature keys
  built from canonical `TypeKey` params/results instead of comparing rendered
  signature strings directly
- runtime structural function identity uses `ConcreteType::Function`
- named function aliases can additionally carry a `TypeId` and `RuntimeTypeInfo`
  entry, so runtime identity can preserve the alias name when present

## Pointer Types

- pointers are first-class runtime values
- pointer identities may be represented structurally through
  `ConcreteType::Pointer` or through a named/runtime `TypeId` when the compiler
  allocates one for a tracked named pointer surface
- reflection-sensitive and method-set-sensitive code relies on this distinction
  so `T` and `*T` remain separable identities

## Runtime `TypeId` Mapping

- built-in runtime kinds occupy fixed low-numbered `TypeId` slots
- user-defined named types allocate higher `TypeId` values during compiler type
  collection
- instantiated generic named types and compiler-tracked pointer identities
  allocate additional `TypeId` values in the same runtime-visible space
- imported package artifacts rebase those `TypeId` ranges when package graphs
  are merged, preserving one stable runtime identity per loaded concrete type

## Cross-Layer Identity Expectations

These invariants should hold for the supported surface:

- parser AST type spellings must round-trip into the compiler's named or
  structural type tables without silently changing meaning
- compiled `ProgramTypeInventory` entries must expose the same named/generic/tag
  identity that reflection and JSON rely on later
- direct runtime identity lookup through `value_runtime_type` must prefer named
  `TypeId` values over anonymous wrapper metadata when both are available
- read-only `reflect.Type` and `reflect.Value` queries must report the same
  type names, field tags, element types, and function signatures the compiled
  inventory describes
- `encoding/json` must read the same field/tag identity that the runtime type
  inventory and reflection layer describe for the supported slice
