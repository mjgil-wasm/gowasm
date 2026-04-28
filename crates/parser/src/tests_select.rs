use super::{parse_source_file, Expr, SelectCase, Stmt, UnaryOp};

#[test]
fn parses_default_only_select() {
    let source = r#"
package main

func main() {
    select {
    default:
        value := 7
    }
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::Select {
            cases: Vec::new(),
            default: Some(vec![Stmt::ShortVarDecl {
                name: "value".into(),
                value: Expr::IntLiteral(7),
            }]),
        }]
    );
}

#[test]
fn parses_select_receive_and_send_cases() {
    let source = r#"
package main

func main() {
    select {
    case <-values:
        first := 1
    case result, ok := <-values:
        second := 2
    case values <- 7:
        third := 3
    default:
        fourth := 4
    }
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::Select {
            cases: vec![
                SelectCase {
                    stmt: Stmt::Expr(Expr::Unary {
                        op: UnaryOp::Receive,
                        expr: Box::new(Expr::Ident("values".into())),
                    }),
                    body: vec![Stmt::ShortVarDecl {
                        name: "first".into(),
                        value: Expr::IntLiteral(1),
                    }],
                },
                SelectCase {
                    stmt: Stmt::ShortVarDeclPair {
                        first: "result".into(),
                        second: "ok".into(),
                        value: Expr::Unary {
                            op: UnaryOp::Receive,
                            expr: Box::new(Expr::Ident("values".into())),
                        },
                    },
                    body: vec![Stmt::ShortVarDecl {
                        name: "second".into(),
                        value: Expr::IntLiteral(2),
                    }],
                },
                SelectCase {
                    stmt: Stmt::Send {
                        chan: Expr::Ident("values".into()),
                        value: Expr::IntLiteral(7),
                    },
                    body: vec![Stmt::ShortVarDecl {
                        name: "third".into(),
                        value: Expr::IntLiteral(3),
                    }],
                },
            ],
            default: Some(vec![Stmt::ShortVarDecl {
                name: "fourth".into(),
                value: Expr::IntLiteral(4),
            }]),
        }]
    );
}
