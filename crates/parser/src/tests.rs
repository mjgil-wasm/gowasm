use super::{
    parse_source_file, AssignTarget, BinaryOp, Expr, Parameter, SourceFile, Stmt, UnaryOp,
};

#[test]
fn parses_basic_program_structure() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Println("hello", 42)
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(file.package_name, "main");
    assert_eq!(file.imports.len(), 1);
    assert_eq!(file.functions.len(), 1);
    assert!(file.functions[0].result_types.is_empty());
}

#[test]
fn parses_import_aliases() {
    let source = r#"
package main
import fs "io/fs"

func main() {}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(file.imports.len(), 1);
    assert_eq!(file.imports[0].alias.as_deref(), Some("fs"));
    assert_eq!(file.imports[0].path, "io/fs");
    assert_eq!(file.imports[0].selector(), "fs");
}

#[test]
fn parses_interface_methods_with_unnamed_parameters() {
    let source = r#"
package main

type Reader interface {
    Read([]byte) (int, error)
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(file.types.len(), 1);
    let reader = &file.types[0];
    let super::TypeDeclKind::Interface { methods, .. } = &reader.kind else {
        panic!("expected interface type");
    };
    assert_eq!(methods.len(), 1);
    assert_eq!(methods[0].name, "Read");
    assert_eq!(methods[0].params.len(), 1);
    assert_eq!(methods[0].params[0].name, "");
    assert_eq!(methods[0].params[0].typ, "[]byte");
}

#[test]
fn parses_range_loops_followed_by_multi_assignment_statements() {
    let source = r#"
package main

func pair() (int, int) {
    return 1, 2
}

func main() {
    entries := []int{1}
    for _, entry := range entries {
        left, right := pair()
        _, _, _ = entry, left, right
    }
}
"#;

    parse_source_file(source).expect("source should parse");
}

#[test]
fn parses_nested_calls_as_expression_statements() {
    let source = r#"
package main
import "fmt"

func helper() {
    fmt.Println("helper")
}

func main() {
    helper()
}
"#;

    let file: SourceFile = parse_source_file(source).expect("source should parse");
    assert_eq!(file.functions.len(), 2);
    assert_eq!(
        file.functions[1].body,
        vec![Stmt::Expr(Expr::Call {
            callee: Box::new(Expr::Ident("helper".into())),
            type_args: Vec::new(),
            args: vec![],
        })]
    );
}

#[test]
fn parses_short_variable_declarations() {
    let source = r#"
package main

func main() {
    label := "hello"
    value := 7
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![
            Stmt::ShortVarDecl {
                name: "label".into(),
                value: Expr::StringLiteral("hello".into()),
            },
            Stmt::ShortVarDecl {
                name: "value".into(),
                value: Expr::IntLiteral(7),
            },
        ]
    );
}

#[test]
fn parses_var_declarations() {
    let source = r#"
package main

func main() {
    var count int
    var label = "hello"
    var ready bool = true
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![
            Stmt::VarDecl {
                name: "count".into(),
                typ: Some("int".into()),
                value: None,
            },
            Stmt::VarDecl {
                name: "label".into(),
                typ: None,
                value: Some(Expr::StringLiteral("hello".into())),
            },
            Stmt::VarDecl {
                name: "ready".into(),
                typ: Some("bool".into()),
                value: Some(Expr::BoolLiteral(true)),
            },
        ]
    );
}

#[test]
fn parses_typed_var_declarations_with_slice_types() {
    let source = r#"
package main

func main() {
    var values []int = []int{1, 2, 3}
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::VarDecl {
            name: "values".into(),
            typ: Some("[]int".into()),
            value: Some(Expr::SliceLiteral {
                element_type: "int".into(),
                elements: vec![
                    Expr::IntLiteral(1),
                    Expr::IntLiteral(2),
                    Expr::IntLiteral(3),
                ],
            }),
        }]
    );
}

#[test]
fn parses_assignment_statements() {
    let source = r#"
package main

func main() {
    value := 7
    value = 8
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![
            Stmt::ShortVarDecl {
                name: "value".into(),
                value: Expr::IntLiteral(7),
            },
            Stmt::Assign {
                target: AssignTarget::Ident("value".into()),
                value: Expr::IntLiteral(8),
            },
        ]
    );
}

#[test]
fn parses_typed_function_parameters() {
    let source = r#"
package main

func greet(name string, count int) {
    helper(name, count)
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].params,
        vec![
            Parameter {
                name: "name".into(),
                typ: "string".into(),
                variadic: false,
            },
            Parameter {
                name: "count".into(),
                typ: "int".into(),
                variadic: false,
            },
        ]
    );
    assert!(file.functions[0].result_types.is_empty());
}

#[test]
fn parses_grouped_function_parameters() {
    let source = r#"
package main

import "fmt"

func Add(a, b int) int {
    return a + b
}

func main() {
    result := Add(5, 7)
    fmt.Printf("5 + 7 = %d\n", result)
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].params,
        vec![
            Parameter {
                name: "a".into(),
                typ: "int".into(),
                variadic: false,
            },
            Parameter {
                name: "b".into(),
                typ: "int".into(),
                variadic: false,
            },
        ]
    );
    assert_eq!(file.functions[0].result_types, vec!["int".to_string()]);
}

#[test]
fn parses_function_result_types_and_return_statements() {
    let source = r#"
package main

func greet(name string) string {
    return name
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(file.functions[0].result_types, vec!["string".to_string()]);
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::Return(vec![Expr::Ident("name".into())])]
    );
}

#[test]
fn parses_addition_with_postfix_precedence() {
    let source = r#"
package main

func main() {
    value := answer() + 2
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "value".into(),
            value: Expr::Binary {
                left: Box::new(Expr::Call {
                    callee: Box::new(Expr::Ident("answer".into())),
                    type_args: Vec::new(),
                    args: vec![],
                }),
                op: BinaryOp::Add,
                right: Box::new(Expr::IntLiteral(2)),
            },
        }]
    );
}

#[test]
fn parses_slice_index_expressions() {
    let source = r#"
package main

func main() {
    value := []int{1, 2, 3}[1]
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "value".into(),
            value: Expr::Index {
                target: Box::new(Expr::SliceLiteral {
                    element_type: "int".into(),
                    elements: vec![
                        Expr::IntLiteral(1),
                        Expr::IntLiteral(2),
                        Expr::IntLiteral(3),
                    ],
                }),
                index: Box::new(Expr::IntLiteral(1)),
            },
        }]
    );
}

#[test]
fn parses_array_literals() {
    let source = r#"
package main

func main() {
    value := [3]int{4, 5, 6}
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "value".into(),
            value: Expr::ArrayLiteral {
                len: 3,
                element_type: "int".into(),
                elements: vec![
                    Expr::IntLiteral(4),
                    Expr::IntLiteral(5),
                    Expr::IntLiteral(6),
                ],
            },
        }]
    );
}

#[test]
fn parses_boolean_literals() {
    let source = r#"
package main

func main() {
    ready := true
    stop := false
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![
            Stmt::ShortVarDecl {
                name: "ready".into(),
                value: Expr::BoolLiteral(true),
            },
            Stmt::ShortVarDecl {
                name: "stop".into(),
                value: Expr::BoolLiteral(false),
            },
        ]
    );
}

#[test]
fn parses_comparisons_after_addition() {
    let source = r#"
package main

func main() {
    ready := answer() + 1 == total + 2
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "ready".into(),
            value: Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::Call {
                        callee: Box::new(Expr::Ident("answer".into())),
                        type_args: Vec::new(),
                        args: vec![],
                    }),
                    op: BinaryOp::Add,
                    right: Box::new(Expr::IntLiteral(1)),
                }),
                op: BinaryOp::Equal,
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::Ident("total".into())),
                    op: BinaryOp::Add,
                    right: Box::new(Expr::IntLiteral(2)),
                }),
            },
        }]
    );
}

#[test]
fn parses_unary_not_and_logical_precedence() {
    let source = r#"
package main

func main() {
    ready := !first || second && third
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "ready".into(),
            value: Expr::Binary {
                left: Box::new(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(Expr::Ident("first".into())),
                }),
                op: BinaryOp::Or,
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::Ident("second".into())),
                    op: BinaryOp::And,
                    right: Box::new(Expr::Ident("third".into())),
                }),
            },
        }]
    );
}

#[test]
fn parses_arithmetic_precedence_and_unary_minus() {
    let source = r#"
package main

func main() {
    value := -left + middle * right / 2
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "value".into(),
            value: Expr::Binary {
                left: Box::new(Expr::Unary {
                    op: UnaryOp::Negate,
                    expr: Box::new(Expr::Ident("left".into())),
                }),
                op: BinaryOp::Add,
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::Binary {
                        left: Box::new(Expr::Ident("middle".into())),
                        op: BinaryOp::Multiply,
                        right: Box::new(Expr::Ident("right".into())),
                    }),
                    op: BinaryOp::Divide,
                    right: Box::new(Expr::IntLiteral(2)),
                }),
            },
        }]
    );
}

#[test]
fn parses_left_shift_with_multiplicative_precedence() {
    let source = r#"
package main

func main() {
    value := 1 + 2 << 3
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "value".into(),
            value: Expr::Binary {
                left: Box::new(Expr::IntLiteral(1)),
                op: BinaryOp::Add,
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::IntLiteral(2)),
                    op: BinaryOp::ShiftLeft,
                    right: Box::new(Expr::IntLiteral(3)),
                }),
            },
        }]
    );
}

#[test]
fn parses_right_shift_with_multiplicative_precedence() {
    let source = r#"
package main

func main() {
    value := 16 >> 2 + 1
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::ShortVarDecl {
            name: "value".into(),
            value: Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::IntLiteral(16)),
                    op: BinaryOp::ShiftRight,
                    right: Box::new(Expr::IntLiteral(2)),
                }),
                op: BinaryOp::Add,
                right: Box::new(Expr::IntLiteral(1)),
            },
        }]
    );
}

#[test]
fn parses_if_else_blocks() {
    let source = r#"
package main

func main() {
    if ready {
        value := 1
    } else {
        value := 2
    }
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::If {
            init: None,
            condition: Expr::Ident("ready".into()),
            then_body: vec![Stmt::ShortVarDecl {
                name: "value".into(),
                value: Expr::IntLiteral(1),
            }],
            else_body: Some(vec![Stmt::ShortVarDecl {
                name: "value".into(),
                value: Expr::IntLiteral(2),
            }]),
        }]
    );
}

#[test]
fn parses_else_if_as_nested_if_statement() {
    let source = r#"
package main

func main() {
    if first {
        value := 1
    } else if second {
        value := 2
    }
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::If {
            init: None,
            condition: Expr::Ident("first".into()),
            then_body: vec![Stmt::ShortVarDecl {
                name: "value".into(),
                value: Expr::IntLiteral(1),
            }],
            else_body: Some(vec![Stmt::If {
                init: None,
                condition: Expr::Ident("second".into()),
                then_body: vec![Stmt::ShortVarDecl {
                    name: "value".into(),
                    value: Expr::IntLiteral(2),
                }],
                else_body: None,
            }]),
        }]
    );
}

#[test]
fn parses_condition_for_loops() {
    let source = r#"
package main

func main() {
    for ready {
        value := 1
    }
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::For {
            init: None,
            condition: Some(Expr::Ident("ready".into())),
            post: None,
            body: vec![Stmt::ShortVarDecl {
                name: "value".into(),
                value: Expr::IntLiteral(1),
            }],
        }]
    );
}

#[test]
fn parses_infinite_for_loops() {
    let source = r#"
package main

func main() {
    for {
        return
    }
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::For {
            init: None,
            condition: None,
            post: None,
            body: vec![Stmt::Return(Vec::new())],
        }]
    );
}

#[test]
fn parses_break_and_continue_statements() {
    let source = r#"
package main

func main() {
    for ready {
        break
        continue
    }
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::For {
            init: None,
            condition: Some(Expr::Ident("ready".into())),
            post: None,
            body: vec![Stmt::Break { label: None }, Stmt::Continue { label: None }],
        }]
    );
}

#[test]
fn parses_increment_and_decrement_statements() {
    let source = r#"
package main

func main() {
    count++
    value--
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![
            Stmt::Increment {
                name: "count".into(),
            },
            Stmt::Decrement {
                name: "value".into(),
            },
        ]
    );
}

#[test]
fn parses_classic_for_clauses() {
    let source = r#"
package main

func main() {
    for i := 0; i < 3; i++ {
        value := i
    }
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::For {
            init: Some(Box::new(Stmt::ShortVarDecl {
                name: "i".into(),
                value: Expr::IntLiteral(0),
            })),
            condition: Some(Expr::Binary {
                left: Box::new(Expr::Ident("i".into())),
                op: BinaryOp::Less,
                right: Box::new(Expr::IntLiteral(3)),
            }),
            post: Some(Box::new(Stmt::Increment { name: "i".into() })),
            body: vec![Stmt::ShortVarDecl {
                name: "value".into(),
                value: Expr::Ident("i".into()),
            }],
        }]
    );
}

#[test]
fn parses_classic_for_post_assignments() {
    let source = r#"
package main

func main() {
    for i := 0; i < 3; i = i + 1 {
        value := i
    }
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.functions[0].body,
        vec![Stmt::For {
            init: Some(Box::new(Stmt::ShortVarDecl {
                name: "i".into(),
                value: Expr::IntLiteral(0),
            })),
            condition: Some(Expr::Binary {
                left: Box::new(Expr::Ident("i".into())),
                op: BinaryOp::Less,
                right: Box::new(Expr::IntLiteral(3)),
            }),
            post: Some(Box::new(Stmt::Assign {
                target: AssignTarget::Ident("i".into()),
                value: Expr::Binary {
                    left: Box::new(Expr::Ident("i".into())),
                    op: BinaryOp::Add,
                    right: Box::new(Expr::IntLiteral(1)),
                },
            })),
            body: vec![Stmt::ShortVarDecl {
                name: "value".into(),
                value: Expr::Ident("i".into()),
            }],
        }]
    );
}
