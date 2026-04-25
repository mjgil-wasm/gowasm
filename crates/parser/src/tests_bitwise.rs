use super::{parse_source_file, BinaryOp, Expr, Stmt, UnaryOp};

#[test]
fn parses_bitwise_or_with_additive_precedence() {
    let source = r#"
package main

func main() {
    value := 1 | 2 + 4
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "value".into(),
            value: Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::IntLiteral(1)),
                    op: BinaryOp::BitOr,
                    right: Box::new(Expr::IntLiteral(2)),
                }),
                op: BinaryOp::Add,
                right: Box::new(Expr::IntLiteral(4)),
            },
        }]
    );
}

#[test]
fn parses_bitwise_and_with_multiplicative_precedence() {
    let source = r#"
package main

func main() {
    value := 1 & 6 + 2
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "value".into(),
            value: Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::IntLiteral(1)),
                    op: BinaryOp::BitAnd,
                    right: Box::new(Expr::IntLiteral(6)),
                }),
                op: BinaryOp::Add,
                right: Box::new(Expr::IntLiteral(2)),
            },
        }]
    );
}

#[test]
fn parses_bitwise_xor_with_additive_precedence() {
    let source = r#"
package main

func main() {
    value := 1 ^ 3 + 4
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "value".into(),
            value: Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::IntLiteral(1)),
                    op: BinaryOp::BitXor,
                    right: Box::new(Expr::IntLiteral(3)),
                }),
                op: BinaryOp::Add,
                right: Box::new(Expr::IntLiteral(4)),
            },
        }]
    );
}

#[test]
fn parses_bitwise_clear_with_multiplicative_precedence() {
    let source = r#"
package main

func main() {
    value := 7 &^ 3 + 1
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "value".into(),
            value: Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::IntLiteral(7)),
                    op: BinaryOp::BitClear,
                    right: Box::new(Expr::IntLiteral(3)),
                }),
                op: BinaryOp::Add,
                right: Box::new(Expr::IntLiteral(1)),
            },
        }]
    );
}

#[test]
fn parses_unary_bitwise_not() {
    let source = r#"
package main

func main() {
    value := ^mask
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "value".into(),
            value: Expr::Unary {
                op: UnaryOp::BitNot,
                expr: Box::new(Expr::Ident("mask".into())),
            },
        }]
    );
}
