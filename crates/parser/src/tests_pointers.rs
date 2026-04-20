use super::{parse_source_file, AssignTarget, Expr, Parameter, Stmt, UnaryOp};

#[test]
fn parses_pointer_type_names() {
    let source = r#"
package main

type Point struct {
    x int
}

var global *Point

func keep(ptr *Point) *Point {
    return ptr
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(file.vars[0].typ.as_deref(), Some("*Point"));
    assert_eq!(
        file.functions[0].params,
        vec![Parameter {
            name: "ptr".into(),
            typ: "*Point".into(),
            variadic: false,
        }]
    );
    assert_eq!(file.functions[0].result_types, vec!["*Point"]);
}

#[test]
fn parses_deref_assignment_targets() {
    let source = r#"
package main

func main() {
    *ptr = 9
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::Assign {
            target: AssignTarget::Deref {
                target: "ptr".into(),
            },
            value: Expr::IntLiteral(9),
        }]
    );
}

#[test]
fn parses_field_address_of_expressions() {
    let source = r#"
package main

func main() {
    ptr := &point.x
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "ptr".into(),
            value: Expr::Unary {
                op: UnaryOp::AddressOf,
                expr: Box::new(Expr::Selector {
                    receiver: Box::new(Expr::Ident("point".into())),
                    field: "x".into(),
                }),
            },
        }]
    );
}

#[test]
fn parses_index_address_of_expressions() {
    let source = r#"
package main

func main() {
    ptr := &values[i]
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "ptr".into(),
            value: Expr::Unary {
                op: UnaryOp::AddressOf,
                expr: Box::new(Expr::Index {
                    target: Box::new(Expr::Ident("values".into())),
                    index: Box::new(Expr::Ident("i".into())),
                }),
            },
        }]
    );
}

#[test]
fn parses_deref_selector_and_index_assignments() {
    let source = r#"
package main

func main() {
    (*ptr).x = 9
    (*items)[i] = 7
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![
            Stmt::Assign {
                target: AssignTarget::DerefSelector {
                    target: "ptr".into(),
                    field: "x".into(),
                },
                value: Expr::IntLiteral(9),
            },
            Stmt::Assign {
                target: AssignTarget::DerefIndex {
                    target: "items".into(),
                    index: Expr::Ident("i".into()),
                },
                value: Expr::IntLiteral(7),
            },
        ]
    );
}

#[test]
fn parses_address_of_and_deref_expressions() {
    let source = r#"
package main

func main() {
    value := 7
    ptr := &value
    copy := *ptr
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![
            Stmt::ShortVarDecl {
                name: "value".into(),
                value: Expr::IntLiteral(7),
            },
            Stmt::ShortVarDecl {
                name: "ptr".into(),
                value: Expr::Unary {
                    op: UnaryOp::AddressOf,
                    expr: Box::new(Expr::Ident("value".into())),
                },
            },
            Stmt::ShortVarDecl {
                name: "copy".into(),
                value: Expr::Unary {
                    op: UnaryOp::Deref,
                    expr: Box::new(Expr::Ident("ptr".into())),
                },
            },
        ]
    );
}
