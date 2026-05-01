use super::{parse_source_file, AssignTarget, Expr, MapLiteralEntry, Stmt};

#[test]
fn parses_typed_var_declarations_with_map_literals() {
    let source = r#"
package main

func main() {
    var scores map[string]int = map[string]int{"go": 1, "wasm": 2}
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::VarDecl {
            name: "scores".into(),
            typ: Some("map[string]int".into()),
            value: Some(Expr::MapLiteral {
                key_type: "string".into(),
                value_type: "int".into(),
                entries: vec![
                    MapLiteralEntry {
                        key: Expr::StringLiteral("go".into()),
                        value: Expr::IntLiteral(1),
                    },
                    MapLiteralEntry {
                        key: Expr::StringLiteral("wasm".into()),
                        value: Expr::IntLiteral(2),
                    },
                ],
            }),
        }]
    );
}

#[test]
fn parses_map_index_expressions() {
    let source = r#"
package main

func main() {
    value := map[string]int{"go": 1}["go"]
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "value".into(),
            value: Expr::Index {
                target: Box::new(Expr::MapLiteral {
                    key_type: "string".into(),
                    value_type: "int".into(),
                    entries: vec![MapLiteralEntry {
                        key: Expr::StringLiteral("go".into()),
                        value: Expr::IntLiteral(1),
                    }],
                }),
                index: Box::new(Expr::StringLiteral("go".into())),
            },
        }]
    );
}

#[test]
fn parses_index_assignment_statements() {
    let source = r#"
package main

func main() {
    values[1] = 9
    scores["go"] = 2
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![
            Stmt::Assign {
                target: AssignTarget::Index {
                    target: "values".into(),
                    index: Expr::IntLiteral(1),
                },
                value: Expr::IntLiteral(9),
            },
            Stmt::Assign {
                target: AssignTarget::Index {
                    target: "scores".into(),
                    index: Expr::StringLiteral("go".into()),
                },
                value: Expr::IntLiteral(2),
            },
        ]
    );
}

#[test]
fn parses_comma_ok_short_var_map_lookups() {
    let source = r#"
package main

func main() {
    value, ok := scores["go"]
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDeclPair {
            first: "value".into(),
            second: "ok".into(),
            value: Expr::Index {
                target: Box::new(Expr::Ident("scores".into())),
                index: Box::new(Expr::StringLiteral("go".into())),
            },
        }]
    );
}

#[test]
fn parses_comma_ok_assignment_map_lookups() {
    let source = r#"
package main

func main() {
    value, ok = scores["go"]
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::AssignPair {
            first: AssignTarget::Ident("value".into()),
            second: AssignTarget::Ident("ok".into()),
            value: Expr::Index {
                target: Box::new(Expr::Ident("scores".into())),
                index: Box::new(Expr::StringLiteral("go".into())),
            },
        }]
    );
}

#[test]
fn parses_nil_map_initializers() {
    let source = r#"
package main

func main() {
    var scores map[string]int = nil
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::VarDecl {
            name: "scores".into(),
            typ: Some("map[string]int".into()),
            value: Some(Expr::NilLiteral),
        }]
    );
}
