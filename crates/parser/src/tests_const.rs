use super::{parse_source_file, Expr, Stmt};

#[test]
fn parses_local_const_declarations() {
    let source = r#"
package main

func main() {
    const answer = 42
    const greeting string = "go"
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![
            Stmt::ConstDecl {
                name: "answer".into(),
                typ: None,
                value: Expr::IntLiteral(42),
                iota: 0,
            },
            Stmt::ConstDecl {
                name: "greeting".into(),
                typ: Some("string".into()),
                value: Expr::StringLiteral("go".into()),
                iota: 0,
            },
        ]
    );
}

#[test]
fn parses_package_const_declarations() {
    let source = r#"
package main

const answer = 42
const greeting string = "go"

func main() {}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.consts,
        vec![
            super::PackageConstDecl {
                name: "answer".into(),
                typ: None,
                value: Expr::IntLiteral(42),
                iota: 0,
            },
            super::PackageConstDecl {
                name: "greeting".into(),
                typ: Some("string".into()),
                value: Expr::StringLiteral("go".into()),
                iota: 0,
            },
        ]
    );
}

#[test]
fn parses_grouped_const_declarations() {
    let source = r#"
package main

const (
    answer = 42
    greeting string = "go"
)

func main() {
    const (
        ready = true
        label = "wasm"
    )
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(file.consts.len(), 2);
    assert_eq!(file.consts[0].iota, 0);
    assert_eq!(file.consts[1].iota, 1);
    assert!(matches!(
        &file.functions[0].body[0],
        Stmt::ConstGroup { decls } if decls.len() == 2 && decls[0].iota == 0 && decls[1].iota == 1
    ));
}

#[test]
fn parses_grouped_consts_with_elided_initializers() {
    let source = r#"
package main

const (
    first = iota
    second
)

func main() {
    const (
        greeting = "go"
        label
    )
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(file.consts[0].value, Expr::Ident("iota".into()));
    assert_eq!(file.consts[1].value, Expr::Ident("iota".into()));
    assert!(matches!(
        &file.functions[0].body[0],
        Stmt::ConstGroup { decls }
            if decls[0].value == Expr::StringLiteral("go".into())
                && decls[1].value == Expr::StringLiteral("go".into())
    ));
}

#[test]
fn reuses_previous_type_for_elided_grouped_consts() {
    let source = r#"
package main

const (
    first string = "go"
    second
)
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(file.consts[0].typ.as_deref(), Some("string"));
    assert_eq!(file.consts[1].typ.as_deref(), Some("string"));
    assert_eq!(file.consts[1].value, Expr::StringLiteral("go".into()));
}
