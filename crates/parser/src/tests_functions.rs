use super::{parse_source_file, Expr, Parameter, Stmt};

#[test]
fn parses_function_literals() {
    let source = r#"
package main

func main() {
    run := func(name string) string {
        return name
    }
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    let Stmt::ShortVarDecl { value, .. } = &file.functions[0].body[0] else {
        panic!("expected short declaration");
    };
    let Expr::FunctionLiteral {
        params,
        result_types,
        body,
    } = value
    else {
        panic!("expected function literal");
    };
    assert_eq!(
        params,
        &vec![Parameter {
            name: "name".into(),
            typ: "string".into(),
            variadic: false,
        }]
    );
    assert_eq!(result_types, &vec!["string".to_string()]);
    assert_eq!(body.len(), 1);
}

#[test]
fn parses_immediately_invoked_function_literals() {
    let source = r#"
package main

func main() {
    func() {
        helper()
    }()
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    let Stmt::Expr(Expr::Call { callee, args, .. }) = &file.functions[0].body[0] else {
        panic!("expected call expression statement");
    };
    assert!(args.is_empty());
    assert!(matches!(callee.as_ref(), Expr::FunctionLiteral { .. }));
}

#[test]
fn parses_source_function_types() {
    let source = r#"
package main

var run func(string) bool

func wrap(fn func(string) bool) func(string) bool {
    return fn
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.vars[0].typ.as_deref(),
        Some("__gowasm_func__(string)->(bool)")
    );
    assert_eq!(
        file.functions[0].params[0].typ,
        "__gowasm_func__(string)->(bool)"
    );
    assert_eq!(
        file.functions[0].result_types,
        vec!["__gowasm_func__(string)->(bool)".to_string()]
    );
}

#[test]
fn parses_multiline_call_arguments_across_inserted_semicolons() {
    let source = r#"
package main

func main() {
    fmt.Println(
        first,
        second
    )
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    let Stmt::Expr(Expr::Call { callee, args, .. }) = &file.functions[0].body[0] else {
        panic!("expected call expression statement");
    };
    assert!(matches!(
        callee.as_ref(),
        Expr::Selector { field, .. } if field == "Println"
    ));
    assert_eq!(
        args,
        &vec![Expr::Ident("first".into()), Expr::Ident("second".into())]
    );
}

#[test]
fn parses_multiline_call_arguments_with_trailing_comma() {
    let source = r#"
package main

func main() {
    fmt.Println(
        first,
        second,
    )
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    let Stmt::Expr(Expr::Call { callee, args, .. }) = &file.functions[0].body[0] else {
        panic!("expected call expression statement");
    };
    assert!(matches!(
        callee.as_ref(),
        Expr::Selector { field, .. } if field == "Println"
    ));
    assert_eq!(
        args,
        &vec![Expr::Ident("first".into()), Expr::Ident("second".into())]
    );
}
