use super::{parse_source_file, Stmt};

#[test]
fn parses_defer_statements() {
    let source = r#"
package main

func main() {
    defer helper(7)
}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert!(matches!(file.functions[0].body[0], Stmt::Defer { .. }));
}
