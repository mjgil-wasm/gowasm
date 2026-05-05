use super::{parse_source_file, Expr, Stmt, SwitchCase, TypeSwitchCase};

#[test]
fn parses_expression_switch_statements() {
    let source = r#"
package main

func main() {
    switch value {
    case 1, 2:
        result := "match"
    default:
        result := "default"
    }
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::Switch {
            init: None,
            expr: Some(Expr::Ident("value".into())),
            cases: vec![SwitchCase {
                expressions: vec![Expr::IntLiteral(1), Expr::IntLiteral(2)],
                body: vec![Stmt::ShortVarDecl {
                    name: "result".into(),
                    value: Expr::StringLiteral("match".into()),
                }],
                fallthrough: false,
            }],
            default: Some(vec![Stmt::ShortVarDecl {
                name: "result".into(),
                value: Expr::StringLiteral("default".into()),
            }]),
            default_index: Some(1),
            default_fallthrough: false,
        }]
    );
}

#[test]
fn parses_expressionless_switch_statements() {
    let source = r#"
package main

func main() {
    switch {
    case ready:
        return
    case fallback:
        value := 1
    }
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::Switch {
            init: None,
            expr: None,
            cases: vec![
                SwitchCase {
                    expressions: vec![Expr::Ident("ready".into())],
                    body: vec![Stmt::Return(Vec::new())],
                    fallthrough: false,
                },
                SwitchCase {
                    expressions: vec![Expr::Ident("fallback".into())],
                    body: vec![Stmt::ShortVarDecl {
                        name: "value".into(),
                        value: Expr::IntLiteral(1),
                    }],
                    fallthrough: false,
                },
            ],
            default: None,
            default_index: None,
            default_fallthrough: false,
        }]
    );
}

#[test]
fn parses_labeled_break_and_continue() {
    let source = r#"
package main
func main() {
    outer:
    for {
        break outer
        continue outer
    }
}
"#;
    let file = parse_source_file(source).expect("should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::Labeled {
            label: "outer".into(),
            stmt: Box::new(Stmt::For {
                init: None,
                condition: None,
                post: None,
                body: vec![
                    Stmt::Break {
                        label: Some("outer".into()),
                    },
                    Stmt::Continue {
                        label: Some("outer".into()),
                    },
                ],
            }),
        }]
    );
}

#[test]
fn parses_switch_default_fallthrough_and_type_switch_init() {
    let source = r#"
package main

func main() {
    switch 9 {
    default:
        result := "default"
        fallthrough
    case 1:
        result := "one"
    }

    switch value := anyValue; typed := value.(type) {
    case int:
        result := typed
    default:
        result := value
    }
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![
            Stmt::Switch {
                init: None,
                expr: Some(Expr::IntLiteral(9)),
                cases: vec![SwitchCase {
                    expressions: vec![Expr::IntLiteral(1)],
                    body: vec![Stmt::ShortVarDecl {
                        name: "result".into(),
                        value: Expr::StringLiteral("one".into()),
                    }],
                    fallthrough: false,
                }],
                default: Some(vec![Stmt::ShortVarDecl {
                    name: "result".into(),
                    value: Expr::StringLiteral("default".into()),
                }]),
                default_index: Some(0),
                default_fallthrough: true,
            },
            Stmt::TypeSwitch {
                init: Some(Box::new(Stmt::ShortVarDecl {
                    name: "value".into(),
                    value: Expr::Ident("anyValue".into()),
                })),
                binding: Some("typed".into()),
                expr: Expr::Ident("value".into()),
                cases: vec![TypeSwitchCase {
                    types: vec!["int".into()],
                    body: vec![Stmt::ShortVarDecl {
                        name: "result".into(),
                        value: Expr::Ident("typed".into()),
                    }],
                }],
                default: Some(vec![Stmt::ShortVarDecl {
                    name: "result".into(),
                    value: Expr::Ident("value".into()),
                }]),
                default_index: Some(1),
            },
        ]
    );
}
