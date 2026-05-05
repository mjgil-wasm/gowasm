use super::{parse_source_file, Expr, InterfaceMethodDecl, Parameter, Stmt, TypeDeclKind};

#[test]
fn parses_empty_interface_type_declarations() {
    let source = r#"
package main

type Any interface {}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(file.types.len(), 1);
    assert_eq!(file.types[0].name, "Any");
    assert_eq!(
        file.types[0].kind,
        TypeDeclKind::Interface {
            methods: Vec::new(),
            embeds: Vec::new(),
        }
    );
}

#[test]
fn parses_type_assertion_expressions() {
    let source = r#"
package main

func main() {
    value := any.(int)
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "value".into(),
            value: Expr::TypeAssert {
                expr: Box::new(Expr::Ident("any".into())),
                asserted_type: "int".into(),
            },
        }]
    );
}

#[test]
fn parses_non_empty_interface_type_declarations() {
    let source = r#"
package main

type Shape interface {
    area() int
    scale(factor int)
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.types[0].kind,
        TypeDeclKind::Interface {
            methods: vec![
                InterfaceMethodDecl {
                    name: "area".into(),
                    params: Vec::new(),
                    result_types: vec!["int".into()],
                },
                InterfaceMethodDecl {
                    name: "scale".into(),
                    params: vec![Parameter {
                        name: "factor".into(),
                        typ: "int".into(),
                        variadic: false,
                    }],
                    result_types: Vec::new(),
                },
            ],
            embeds: Vec::new(),
        }
    );
}

#[test]
fn parses_comma_ok_type_assertion_forms() {
    let source = r#"
package main

func main() {
    value, ok := any.(int)
    value, ok = any.(int)
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![
            Stmt::ShortVarDeclPair {
                first: "value".into(),
                second: "ok".into(),
                value: Expr::TypeAssert {
                    expr: Box::new(Expr::Ident("any".into())),
                    asserted_type: "int".into(),
                },
            },
            Stmt::AssignPair {
                first: super::AssignTarget::Ident("value".into()),
                second: super::AssignTarget::Ident("ok".into()),
                value: Expr::TypeAssert {
                    expr: Box::new(Expr::Ident("any".into())),
                    asserted_type: "int".into(),
                },
            },
        ]
    );
}

#[test]
fn parses_interface_embedding() {
    let source = r#"
package main

type Reader interface {
    Read(buf []byte) (int, error)
}

type Writer interface {
    Write(buf []byte) (int, error)
}

type ReadWriter interface {
    Reader
    Writer
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(file.types.len(), 3);
    assert_eq!(
        file.types[2].kind,
        TypeDeclKind::Interface {
            methods: Vec::new(),
            embeds: vec!["Reader".into(), "Writer".into()],
        }
    );
}
