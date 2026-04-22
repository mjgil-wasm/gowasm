use super::{parse_source_file, Expr, Stmt};

#[test]
fn parses_make_map_expressions() {
    let source = r#"
package main

func main() {
    values := make(map[string]int)
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "values".into(),
            value: Expr::Make {
                type_name: "map[string]int".into(),
                args: vec![],
            },
        }]
    );
}

#[test]
fn parses_make_slice_expressions() {
    let source = r#"
package main

func main() {
    values := make([]int, 3)
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "values".into(),
            value: Expr::Make {
                type_name: "[]int".into(),
                args: vec![Expr::IntLiteral(3)],
            },
        }]
    );
}

#[test]
fn parses_make_channel_expressions() {
    let source = r#"
package main

func main() {
    values := make(chan int)
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "values".into(),
            value: Expr::Make {
                type_name: "chan int".into(),
                args: vec![],
            },
        }]
    );
}

#[test]
fn parses_buffered_make_channel_expressions() {
    let source = r#"
package main

func main() {
    values := make(chan int, 2)
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "values".into(),
            value: Expr::Make {
                type_name: "chan int".into(),
                args: vec![Expr::IntLiteral(2)],
            },
        }]
    );
}

#[test]
fn parses_receive_only_make_channel_expressions() {
    let source = r#"
package main

func main() {
    values := make(<-chan int)
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "values".into(),
            value: Expr::Make {
                type_name: "<-chan int".into(),
                args: vec![],
            },
        }]
    );
}

#[test]
fn parses_send_only_make_channel_expressions() {
    let source = r#"
package main

func main() {
    values := make(chan<- int)
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "values".into(),
            value: Expr::Make {
                type_name: "chan<- int".into(),
                args: vec![],
            },
        }]
    );
}
