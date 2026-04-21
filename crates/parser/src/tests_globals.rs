use super::{parse_source_file, Expr, PackageVarDecl};

#[test]
fn parses_package_level_vars() {
    let source = r#"
package main

var count int
var label = "hi"

func main() {}
"#;

    let file = parse_source_file(source).expect("source should parse");
    assert_eq!(
        file.vars,
        vec![
            PackageVarDecl {
                name: "count".into(),
                typ: Some("int".into()),
                value: None,
            },
            PackageVarDecl {
                name: "label".into(),
                typ: None,
                value: Some(Expr::StringLiteral("hi".into())),
            },
        ]
    );
}
