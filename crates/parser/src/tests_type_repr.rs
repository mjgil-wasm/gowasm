use super::{parse_source_file, parse_type_repr, TypeChannelDirection, TypeDeclKind, TypeRepr};

#[test]
fn parses_nested_type_repr_combinations() {
    let typ = parse_type_repr(
        "func(map[string]*Pair[int], <-chan []Value[T]) (chan<- map[string]func(*Node[T]) []Result[U], [3]*pkg.Item)",
    )
    .expect("type should parse");

    assert_eq!(
        typ.render(),
        "__gowasm_func__(map[string]*Pair[int],<-chan []Value[T])->(chan<- map[string]__gowasm_func__(*Node[T])->([]Result[U]),[3]*pkg.Item)"
    );

    match typ {
        TypeRepr::Function { params, results } => {
            assert_eq!(params.len(), 2);
            assert_eq!(results.len(), 2);
            assert!(matches!(
                &params[0],
                TypeRepr::Map {
                    key,
                    value,
                } if matches!(&**key, TypeRepr::Name(name) if name == "string")
                    && matches!(&**value, TypeRepr::Pointer(_))
            ));
            assert!(matches!(
                &params[1],
                TypeRepr::Channel {
                    direction: TypeChannelDirection::ReceiveOnly,
                    element,
                } if matches!(&**element, TypeRepr::Slice(_))
            ));
            assert!(matches!(
                &results[0],
                TypeRepr::Channel {
                    direction: TypeChannelDirection::SendOnly,
                    element,
                } if matches!(&**element, TypeRepr::Map { .. })
            ));
            assert!(matches!(
                &results[1],
                TypeRepr::Array { len: 3, element } if matches!(&**element, TypeRepr::Pointer(_))
            ));
        }
        other => panic!("expected function type, got {other:?}"),
    }
}

#[test]
fn source_file_type_fields_render_from_canonical_repr() {
    let source = r#"
package main

type Handler func([]map[string]Pair[int], chan<- *Result[T]) <-chan func([2]Pair[int]) *Value[T]

type Registry struct {
    Build Factory[func(*pkg.Node[T]) []Result[U], map[string]Value[V]]
}
"#;

    let parsed = parse_source_file(source).expect("source should parse");
    match &parsed.types[0].kind {
        TypeDeclKind::Alias { underlying } => assert_eq!(
            underlying,
            "__gowasm_func__([]map[string]Pair[int],chan<- *Result[T])->(<-chan __gowasm_func__([2]Pair[int])->(*Value[T]))"
        ),
        other => panic!("expected alias type, got {other:?}"),
    }
    match &parsed.types[1].kind {
        TypeDeclKind::Struct { fields } => assert_eq!(
            fields[0].typ,
            "Factory[__gowasm_func__(*pkg.Node[T])->([]Result[U]),map[string]Value[V]]"
        ),
        other => panic!("expected struct type, got {other:?}"),
    }
}

#[test]
fn parses_anonymous_empty_struct_type_repr() {
    let typ = parse_type_repr("chan struct{}").expect("type should parse");
    assert_eq!(typ.render(), "chan struct{}");

    match typ {
        TypeRepr::Channel {
            direction: TypeChannelDirection::Bidirectional,
            element,
        } => assert!(matches!(&*element, TypeRepr::Struct { fields } if fields.is_empty())),
        other => panic!("expected channel type, got {other:?}"),
    }
}

#[test]
fn parses_anonymous_struct_function_parameter_type_repr() {
    let typ = parse_type_repr("func(t *testing.T, row struct{name string; expected int})")
        .expect("type should parse");
    assert_eq!(
        typ.render(),
        "__gowasm_func__(*testing.T,struct{name string;expected int})->()"
    );
}
