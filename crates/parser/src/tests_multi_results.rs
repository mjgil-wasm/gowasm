use super::{parse_source_file, Expr, Stmt};

#[test]
fn parses_multi_result_function_signatures_and_returns() {
    let source = r#"
package main

func pair() (int, string) {
    return 7, "go"
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].result_types,
        vec!["int".to_string(), "string".to_string()]
    );
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::Return(vec![
            Expr::IntLiteral(7),
            Expr::StringLiteral("go".into()),
        ])]
    );
}

#[test]
fn parses_multi_result_interface_methods() {
    let source = r#"
package main

type Reader interface {
    next() (string, bool)
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    let method = match &file.types[0].kind {
        super::TypeDeclKind::Interface { methods, .. } => &methods[0],
        other => panic!("expected interface, found {other:?}"),
    };
    assert_eq!(method.name, "next");
    assert_eq!(
        method.result_types,
        vec!["string".to_string(), "bool".to_string()]
    );
}

#[test]
fn parses_triple_result_short_declarations_and_assignments() {
    let source = r#"
package main

func main() {
    first, second, third := triple()
    first, second, third = triple()
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![
            Stmt::ShortVarDeclTriple {
                first: "first".into(),
                second: "second".into(),
                third: "third".into(),
                value: Expr::Call {
                    callee: Box::new(Expr::Ident("triple".into())),
                    type_args: Vec::new(),
                    args: vec![],
                },
            },
            Stmt::AssignTriple {
                first: super::AssignTarget::Ident("first".into()),
                second: super::AssignTarget::Ident("second".into()),
                third: super::AssignTarget::Ident("third".into()),
                value: Expr::Call {
                    callee: Box::new(Expr::Ident("triple".into())),
                    type_args: Vec::new(),
                    args: vec![],
                },
            },
        ]
    );
}

#[test]
fn parses_quad_result_short_declarations_and_assignments() {
    let source = r#"
package main

func main() {
    first, second, third, fourth := quad()
    first, second, third, fourth = quad()
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![
            Stmt::ShortVarDeclQuad {
                first: "first".into(),
                second: "second".into(),
                third: "third".into(),
                fourth: "fourth".into(),
                value: Expr::Call {
                    callee: Box::new(Expr::Ident("quad".into())),
                    type_args: Vec::new(),
                    args: vec![],
                },
            },
            Stmt::AssignQuad {
                first: super::AssignTarget::Ident("first".into()),
                second: super::AssignTarget::Ident("second".into()),
                third: super::AssignTarget::Ident("third".into()),
                fourth: super::AssignTarget::Ident("fourth".into()),
                value: Expr::Call {
                    callee: Box::new(Expr::Ident("quad".into())),
                    type_args: Vec::new(),
                    args: vec![],
                },
            },
        ]
    );
}

#[test]
fn parses_blank_identifier_multi_result_statements() {
    let source = r#"
package main

func main() {
    _, word := pair()
    _, _, tail, err := quad()
    _, word = pair()
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![
            Stmt::ShortVarDeclPair {
                first: "_".into(),
                second: "word".into(),
                value: Expr::Call {
                    callee: Box::new(Expr::Ident("pair".into())),
                    type_args: Vec::new(),
                    args: vec![],
                },
            },
            Stmt::ShortVarDeclQuad {
                first: "_".into(),
                second: "_".into(),
                third: "tail".into(),
                fourth: "err".into(),
                value: Expr::Call {
                    callee: Box::new(Expr::Ident("quad".into())),
                    type_args: Vec::new(),
                    args: vec![],
                },
            },
            Stmt::AssignPair {
                first: super::AssignTarget::Ident("_".into()),
                second: super::AssignTarget::Ident("word".into()),
                value: Expr::Call {
                    callee: Box::new(Expr::Ident("pair".into())),
                    type_args: Vec::new(),
                    args: vec![],
                },
            },
        ]
    );
}

#[test]
fn parses_assignment_expression_lists() {
    let source = r#"
package main

func main() {
    first, second := left(), right()
    first, second = nextLeft(), nextRight()
    only := left(), right()
    only = nextLeft(), nextRight()
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![
            Stmt::ShortVarDeclList {
                names: vec!["first".into(), "second".into()],
                values: vec![
                    Expr::Call {
                        callee: Box::new(Expr::Ident("left".into())),
                        type_args: Vec::new(),
                        args: vec![],
                    },
                    Expr::Call {
                        callee: Box::new(Expr::Ident("right".into())),
                        type_args: Vec::new(),
                        args: vec![],
                    },
                ],
            },
            Stmt::AssignList {
                targets: vec![
                    super::AssignTarget::Ident("first".into()),
                    super::AssignTarget::Ident("second".into()),
                ],
                values: vec![
                    Expr::Call {
                        callee: Box::new(Expr::Ident("nextLeft".into())),
                        type_args: Vec::new(),
                        args: vec![],
                    },
                    Expr::Call {
                        callee: Box::new(Expr::Ident("nextRight".into())),
                        type_args: Vec::new(),
                        args: vec![],
                    },
                ],
            },
            Stmt::ShortVarDeclList {
                names: vec!["only".into()],
                values: vec![
                    Expr::Call {
                        callee: Box::new(Expr::Ident("left".into())),
                        type_args: Vec::new(),
                        args: vec![],
                    },
                    Expr::Call {
                        callee: Box::new(Expr::Ident("right".into())),
                        type_args: Vec::new(),
                        args: vec![],
                    },
                ],
            },
            Stmt::AssignList {
                targets: vec![super::AssignTarget::Ident("only".into())],
                values: vec![
                    Expr::Call {
                        callee: Box::new(Expr::Ident("nextLeft".into())),
                        type_args: Vec::new(),
                        args: vec![],
                    },
                    Expr::Call {
                        callee: Box::new(Expr::Ident("nextRight".into())),
                        type_args: Vec::new(),
                        args: vec![],
                    },
                ],
            },
        ]
    );
}

#[test]
fn parses_assignment_expression_lists_with_tail_multi_results() {
    let source = r#"
package main

func main() {
    label, number, word := prefix(), pair()
    label, number, word = nextPrefix(), pair()
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![
            Stmt::ShortVarDeclList {
                names: vec!["label".into(), "number".into(), "word".into()],
                values: vec![
                    Expr::Call {
                        callee: Box::new(Expr::Ident("prefix".into())),
                        type_args: Vec::new(),
                        args: vec![],
                    },
                    Expr::Call {
                        callee: Box::new(Expr::Ident("pair".into())),
                        type_args: Vec::new(),
                        args: vec![],
                    },
                ],
            },
            Stmt::AssignList {
                targets: vec![
                    super::AssignTarget::Ident("label".into()),
                    super::AssignTarget::Ident("number".into()),
                    super::AssignTarget::Ident("word".into()),
                ],
                values: vec![
                    Expr::Call {
                        callee: Box::new(Expr::Ident("nextPrefix".into())),
                        type_args: Vec::new(),
                        args: vec![],
                    },
                    Expr::Call {
                        callee: Box::new(Expr::Ident("pair".into())),
                        type_args: Vec::new(),
                        args: vec![],
                    },
                ],
            },
        ]
    );
}
