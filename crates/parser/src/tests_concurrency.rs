use super::{parse_source_file, Expr, Stmt, UnaryOp};

#[test]
fn parses_go_statement_calls() {
    let source = r#"
package main

func worker() {}

func main() {
    go worker()
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    let body = &file.functions[1].body;
    assert_eq!(
        body,
        &[Stmt::Go {
            call: Expr::Call {
                callee: Box::new(Expr::Ident("worker".into())),
                type_args: Vec::new(),
                args: Vec::new(),
            },
        }]
    );
}

#[test]
fn parses_send_and_receive_channel_syntax() {
    let source = r#"
package main

func main() {
    values <- 7
    result := <-values
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    let body = &file.functions[0].body;
    assert_eq!(
        body,
        &[
            Stmt::Send {
                chan: Expr::Ident("values".into()),
                value: Expr::IntLiteral(7),
            },
            Stmt::ShortVarDecl {
                name: "result".into(),
                value: Expr::Unary {
                    op: UnaryOp::Receive,
                    expr: Box::new(Expr::Ident("values".into())),
                },
            },
        ]
    );
}

#[test]
fn parses_comma_ok_receive_forms() {
    let source = r#"
package main

func main() {
    value, ok := <-values
    value, ok = <-values
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    let receive = Expr::Unary {
        op: UnaryOp::Receive,
        expr: Box::new(Expr::Ident("values".into())),
    };
    assert_eq!(
        file.functions[0].body,
        vec![
            Stmt::ShortVarDeclPair {
                first: "value".into(),
                second: "ok".into(),
                value: receive.clone(),
            },
            Stmt::AssignPair {
                first: super::AssignTarget::Ident("value".into()),
                second: super::AssignTarget::Ident("ok".into()),
                value: receive,
            },
        ]
    );
}
