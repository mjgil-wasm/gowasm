use crate::{parse_source_file, Expr, Stmt, TypeParam};

#[test]
fn parses_generic_function_with_single_type_param() {
    let source = r#"
        package main
        func Identity[T any](x T) T {
            return x
        }
        func main() {}
    "#;
    let file = parse_source_file(source).unwrap();
    let identity = &file.functions[0];
    assert_eq!(identity.name, "Identity");
    assert_eq!(
        identity.type_params,
        vec![TypeParam {
            name: "T".into(),
            constraint: "interface{}".into(),
        }]
    );
    assert_eq!(identity.params.len(), 1);
    assert_eq!(identity.params[0].typ, "T");
    assert_eq!(identity.result_types, vec!["T".to_string()]);
}

#[test]
fn parses_generic_function_with_multiple_type_params() {
    let source = r#"
        package main
        func Pair[A any, B comparable](a A, b B) A {
            return a
        }
        func main() {}
    "#;
    let file = parse_source_file(source).unwrap();
    let pair = &file.functions[0];
    assert_eq!(pair.name, "Pair");
    assert_eq!(
        pair.type_params,
        vec![
            TypeParam {
                name: "A".into(),
                constraint: "interface{}".into(),
            },
            TypeParam {
                name: "B".into(),
                constraint: "comparable".into(),
            },
        ]
    );
}

#[test]
fn parses_generic_struct_type() {
    let source = r#"
        package main
        type Box[T any] struct {
            value T
        }
        func main() {}
    "#;
    let file = parse_source_file(source).unwrap();
    let box_type = &file.types[0];
    assert_eq!(box_type.name, "Box");
    assert_eq!(
        box_type.type_params,
        vec![TypeParam {
            name: "T".into(),
            constraint: "interface{}".into(),
        }]
    );
}

#[test]
fn parses_non_generic_function_has_empty_type_params() {
    let source = r#"
        package main
        func Add(a int, b int) int {
            return a
        }
        func main() {}
    "#;
    let file = parse_source_file(source).unwrap();
    assert!(file.functions[0].type_params.is_empty());
}

#[test]
fn parses_non_generic_type_has_empty_type_params() {
    let source = r#"
        package main
        type Point struct {
            X int
            Y int
        }
        func main() {}
    "#;
    let file = parse_source_file(source).unwrap();
    assert!(file.types[0].type_params.is_empty());
}

#[test]
fn parses_generic_type_alias() {
    let source = r#"
        package main
        type MySlice[T any] []T
        func main() {}
    "#;
    let file = parse_source_file(source).unwrap();
    let alias = &file.types[0];
    assert_eq!(alias.name, "MySlice");
    assert_eq!(
        alias.type_params,
        vec![TypeParam {
            name: "T".into(),
            constraint: "interface{}".into(),
        }]
    );
}

#[test]
fn parses_generic_function_with_interface_constraint() {
    let source = r#"
        package main
        type Stringer interface {
            String() string
        }
        func Print[T Stringer](x T) string {
            return x
        }
        func main() {}
    "#;
    let file = parse_source_file(source).unwrap();
    let print_fn = &file.functions[0];
    assert_eq!(
        print_fn.type_params,
        vec![TypeParam {
            name: "T".into(),
            constraint: "Stringer".into(),
        }]
    );
}

#[test]
fn parses_generic_function_with_inline_interface_constraint() {
    let source = r#"
        package main
        func Format[T interface {
            comparable
            String() string
            int | string
        }](x T) string {
            return x.String()
        }
        func main() {}
    "#;

    let file = parse_source_file(source).unwrap();
    let format_fn = &file.functions[0];
    assert_eq!(
        format_fn.type_params,
        vec![TypeParam {
            name: "T".into(),
            constraint: "interface{String() string;comparable;int|string}".into(),
        }]
    );
}

#[test]
fn parses_generic_type_with_embedded_interface_constraint() {
    let source = r#"
        package main
        type Stringer interface {
            String() string
        }
        type Box[T interface { Stringer; comparable }] struct {
            value T
        }
        func main() {}
    "#;

    let file = parse_source_file(source).unwrap();
    assert_eq!(
        file.types[1].type_params,
        vec![TypeParam {
            name: "T".into(),
            constraint: "interface{Stringer;comparable}".into(),
        }]
    );
}

#[test]
fn parses_explicit_type_argument_calls() {
    let source = r#"
        package main
        func Pair[A any, B comparable](a A, b B) A { return a }
        func main() {
            Pair[int, string](1, "x")
        }
    "#;
    let file = parse_source_file(source).unwrap();
    let Stmt::Expr(Expr::Call {
        callee,
        type_args,
        args,
    }) = &file.functions[1].body[0]
    else {
        panic!("expected explicit generic call");
    };
    assert!(matches!(callee.as_ref(), Expr::Ident(name) if name == "Pair"));
    assert_eq!(type_args, &vec!["int".to_string(), "string".to_string()]);
    assert_eq!(
        args,
        &vec![Expr::IntLiteral(1), Expr::StringLiteral("x".into())]
    );
}

#[test]
fn preserves_index_call_parsing_when_brackets_are_not_type_args() {
    let source = r#"
        package main
        func main() {
            callbacks[i](1)
        }
    "#;
    let file = parse_source_file(source).unwrap();
    let Stmt::Expr(Expr::Call {
        callee,
        type_args,
        args,
    }) = &file.functions[0].body[0]
    else {
        panic!("expected call expression");
    };
    assert!(type_args.is_empty());
    assert!(matches!(
        callee.as_ref(),
        Expr::Index { target, index }
            if matches!(target.as_ref(), Expr::Ident(name) if name == "callbacks")
                && matches!(index.as_ref(), Expr::Ident(name) if name == "i")
    ));
    assert_eq!(args, &vec![Expr::IntLiteral(1)]);
}

#[test]
fn parses_generic_type_references_in_type_positions() {
    let source = r#"
        package main
        type Box[T any] struct {
            value T
        }
        type BoxList[T any] []Box[T]
        func (b Box[T]) Wrap(items []Box[T]) Box[T] {
            return b
        }
    "#;
    let file = parse_source_file(source).unwrap();
    assert_eq!(
        file.types[1].kind,
        crate::TypeDeclKind::Alias {
            underlying: "[]Box[T]".into(),
        }
    );
    let receiver = file.functions[0]
        .receiver
        .as_ref()
        .expect("method receiver");
    assert_eq!(receiver.typ, "Box[T]");
    assert_eq!(file.functions[0].params[0].typ, "[]Box[T]");
    assert_eq!(file.functions[0].result_types, vec!["Box[T]".to_string()]);
}
