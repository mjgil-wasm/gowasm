use super::{parse_source_file, Expr, Stmt};

#[test]
fn parses_range_loops_with_key_and_value_bindings() {
    let source = r#"
package main

func main() {
    for index, value := range values {
        use(index, value)
    }
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::RangeFor {
            key: "index".into(),
            value: Some("value".into()),
            assign: false,
            expr: Expr::Ident("values".into()),
            body: vec![Stmt::Expr(Expr::Call {
                callee: Box::new(Expr::Ident("use".into())),
                type_args: Vec::new(),
                args: vec![Expr::Ident("index".into()), Expr::Ident("value".into())],
            })],
        }]
    );
}

#[test]
fn parses_range_loops_with_only_a_key_binding() {
    let source = r#"
package main

func main() {
    for key := range values {
        use(key)
    }
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::RangeFor {
            key: "key".into(),
            value: None,
            assign: false,
            expr: Expr::Ident("values".into()),
            body: vec![Stmt::Expr(Expr::Call {
                callee: Box::new(Expr::Ident("use".into())),
                type_args: Vec::new(),
                args: vec![Expr::Ident("key".into())],
            })],
        }]
    );
}

#[test]
fn parses_range_loops_with_assignment_bindings() {
    let source = r#"
package main

func main() {
    for index, value = range values {
        use(index, value)
    }
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::RangeFor {
            key: "index".into(),
            value: Some("value".into()),
            assign: true,
            expr: Expr::Ident("values".into()),
            body: vec![Stmt::Expr(Expr::Call {
                callee: Box::new(Expr::Ident("use".into())),
                type_args: Vec::new(),
                args: vec![Expr::Ident("index".into()), Expr::Ident("value".into())],
            })],
        }]
    );
}

#[test]
fn parses_range_loops_without_bindings() {
    let source = r#"
package main

func main() {
    for range values {
        use()
    }
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::RangeFor {
            key: "_".into(),
            value: None,
            assign: false,
            expr: Expr::Ident("values".into()),
            body: vec![Stmt::Expr(Expr::Call {
                callee: Box::new(Expr::Ident("use".into())),
                type_args: Vec::new(),
                args: vec![],
            })],
        }]
    );
}

#[test]
fn parses_range_loops_with_empty_bodies_over_identifier_expressions() {
    let source = r#"
package main

func main() {
    for index, value := range values {
    }
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::RangeFor {
            key: "index".into(),
            value: Some("value".into()),
            assign: false,
            expr: Expr::Ident("values".into()),
            body: vec![],
        }]
    );
}
