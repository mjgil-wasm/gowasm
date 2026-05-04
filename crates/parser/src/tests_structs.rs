use super::{
    parse_source_file, AssignTarget, Expr, SourceFile, Stmt, StructLiteralField, TypeDeclKind,
    TypeFieldDecl,
};

#[test]
fn parses_struct_type_declarations() {
    let source = r#"
package main

type Point struct {
    x int
    y int
}

func main() {}
"#;

    let file: SourceFile = parse_source_file(source).expect("source should parse");
    assert_eq!(file.types.len(), 1);
    assert_eq!(
        file.types[0].kind,
        TypeDeclKind::Struct {
            fields: vec![
                TypeFieldDecl {
                    name: "x".into(),
                    typ: "int".into(),
                    embedded: false,
                    tag: None,
                },
                TypeFieldDecl {
                    name: "y".into(),
                    typ: "int".into(),
                    embedded: false,
                    tag: None,
                },
            ],
        }
    );
}

#[test]
fn parses_struct_field_tags() {
    let source = r#"
package main

type Inner struct{}
type Box struct{}

type Config struct {
    Name string `json:"name"`
    Inner `json:"inner"`
    *Box `json:"box"`
}

func main() {}
"#;

    let file: SourceFile = parse_source_file(source).expect("source should parse");
    assert_eq!(file.types.len(), 3);
    assert_eq!(
        file.types[2].kind,
        TypeDeclKind::Struct {
            fields: vec![
                TypeFieldDecl {
                    name: "Name".into(),
                    typ: "string".into(),
                    embedded: false,
                    tag: Some(r#"json:"name""#.into()),
                },
                TypeFieldDecl {
                    name: "Inner".into(),
                    typ: "Inner".into(),
                    embedded: true,
                    tag: Some(r#"json:"inner""#.into()),
                },
                TypeFieldDecl {
                    name: "Box".into(),
                    typ: "*Box".into(),
                    embedded: true,
                    tag: Some(r#"json:"box""#.into()),
                },
            ],
        }
    );
}

#[test]
fn parses_struct_literals() {
    let source = r#"
package main

type Point struct {
    x int
    y int
}

func main() {
    point := Point{x: 1, y: 2}
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "point".into(),
            value: Expr::StructLiteral {
                type_name: "Point".into(),
                fields: vec![
                    StructLiteralField {
                        name: "x".into(),
                        value: Expr::IntLiteral(1),
                    },
                    StructLiteralField {
                        name: "y".into(),
                        value: Expr::IntLiteral(2),
                    },
                ],
            },
        }]
    );
}

#[test]
fn parses_imported_struct_literals() {
    let source = r#"
package main

import "example.com/lib"

func main() {
    value := lib.Point{X: 1, Y: 2}
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "value".into(),
            value: Expr::StructLiteral {
                type_name: "lib.Point".into(),
                fields: vec![
                    StructLiteralField {
                        name: "X".into(),
                        value: Expr::IntLiteral(1),
                    },
                    StructLiteralField {
                        name: "Y".into(),
                        value: Expr::IntLiteral(2),
                    },
                ],
            },
        }]
    );
}

#[test]
fn parses_struct_field_selectors() {
    let source = r#"
package main

type Point struct {
    x int
}

func main() {
    value := Point{x: 1}.x
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "value".into(),
            value: Expr::Selector {
                receiver: Box::new(Expr::StructLiteral {
                    type_name: "Point".into(),
                    fields: vec![StructLiteralField {
                        name: "x".into(),
                        value: Expr::IntLiteral(1),
                    }],
                }),
                field: "x".into(),
            },
        }]
    );
}

#[test]
fn parses_struct_field_assignments() {
    let source = r#"
package main

type Point struct {
    x int
}

func main() {
    point := Point{x: 1}
    point.x = 3
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![
            Stmt::ShortVarDecl {
                name: "point".into(),
                value: Expr::StructLiteral {
                    type_name: "Point".into(),
                    fields: vec![StructLiteralField {
                        name: "x".into(),
                        value: Expr::IntLiteral(1),
                    }],
                },
            },
            Stmt::Assign {
                target: AssignTarget::Selector {
                    receiver: "point".into(),
                    field: "x".into(),
                },
                value: Expr::IntLiteral(3),
            },
        ]
    );
}

#[test]
fn parses_value_receiver_methods() {
    let source = r#"
package main

type Point struct {
    x int
    y int
}

func (point Point) sum() int {
    return point.x + point.y
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].receiver,
        Some(super::Parameter {
            name: "point".into(),
            typ: "Point".into(),
            variadic: false,
        })
    );
    assert_eq!(file.functions[0].name, "sum");
    assert_eq!(file.functions[0].result_types, vec!["int".to_string()]);
}

#[test]
fn parses_unnamed_value_receiver_methods() {
    let source = r#"
package main

type Greeter struct{}

func (Greeter) run() {}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].receiver,
        Some(super::Parameter {
            name: String::new(),
            typ: "Greeter".into(),
            variadic: false,
        })
    );
    assert_eq!(file.functions[0].name, "run");
}
