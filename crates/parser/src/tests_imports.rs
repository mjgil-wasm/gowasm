use gowasm_lexer::Span;

use super::parse_source_file_with_spans;

#[test]
fn records_import_spans_for_single_and_grouped_imports() {
    let source = r#"
package main

import "fmt"
import (
    "strings"
    "example.com/app/lib"
)

func main() {}
"#;

    let (_file, spans) = parse_source_file_with_spans(source).expect("source should parse");

    assert_eq!(
        import_substrings(source, &spans.imports),
        vec!["\"fmt\"", "\"strings\"", "\"example.com/app/lib\""]
    );
}

fn import_substrings<'a>(source: &'a str, spans: &[Span]) -> Vec<&'a str> {
    spans
        .iter()
        .map(|span| &source[span.start..span.end])
        .collect()
}
