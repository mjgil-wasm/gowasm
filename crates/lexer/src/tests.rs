use super::{lex, TokenKind};

#[test]
fn lexes_a_basic_source_file() {
    let source = r#"
package main
import "fmt"

func main() {
    fmt.Println("hello", 42)
}
"#;

    let tokens = lex(source).expect("source should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Package,
            TokenKind::Ident("main".into()),
            TokenKind::Semicolon,
            TokenKind::Import,
            TokenKind::String("fmt".into()),
            TokenKind::Semicolon,
            TokenKind::Func,
            TokenKind::Ident("main".into()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::Ident("fmt".into()),
            TokenKind::Dot,
            TokenKind::Ident("Println".into()),
            TokenKind::LParen,
            TokenKind::String("hello".into()),
            TokenKind::Comma,
            TokenKind::Int(42),
            TokenKind::RParen,
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn inserts_semicolons_after_newlines_between_statements() {
    let source = r#"
package main

func main() {
    ptr := &point.x
    *ptr = 9
}
"#;

    let kinds: Vec<TokenKind> = lex(source)
        .expect("source should lex")
        .into_iter()
        .map(|token| token.kind)
        .collect();

    assert_eq!(
        kinds,
        vec![
            TokenKind::Package,
            TokenKind::Ident("main".into()),
            TokenKind::Semicolon,
            TokenKind::Func,
            TokenKind::Ident("main".into()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::Ident("ptr".into()),
            TokenKind::ColonEqual,
            TokenKind::BitAnd,
            TokenKind::Ident("point".into()),
            TokenKind::Dot,
            TokenKind::Ident("x".into()),
            TokenKind::Semicolon,
            TokenKind::Star,
            TokenKind::Ident("ptr".into()),
            TokenKind::Equal,
            TokenKind::Int(9),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn ignores_line_comments() {
    let source = "// comment\npackage main";
    let tokens = lex(source).expect("comments should be ignored");
    assert_eq!(tokens[0].kind, TokenKind::Package);
}

#[test]
fn decodes_supported_string_escapes() {
    let tokens = lex(r#""line\nbreak""#).expect("string should lex");
    assert_eq!(tokens[0].kind, TokenKind::String("line\nbreak".into()));
}

#[test]
fn preserves_utf8_string_literals() {
    let tokens = lex(r#""hé""#).expect("utf8 string should lex");
    assert_eq!(tokens[0].kind, TokenKind::String("hé".into()));
}

#[test]
fn lexes_short_variable_declarations() {
    let tokens = lex("value := 7").expect("short declaration should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("value".into()),
            TokenKind::ColonEqual,
            TokenKind::Int(7),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_var_declarations() {
    let tokens = lex("var value int = 7").expect("var declaration should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Var,
            TokenKind::Ident("value".into()),
            TokenKind::Ident("int".into()),
            TokenKind::Equal,
            TokenKind::Int(7),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_const_declarations() {
    let tokens = lex("const answer int = 42").expect("const declaration should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Const,
            TokenKind::Ident("answer".into()),
            TokenKind::Ident("int".into()),
            TokenKind::Equal,
            TokenKind::Int(42),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_collection_brackets() {
    let tokens = lex("[]int{1}[0]").expect("collection brackets should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::LBracket,
            TokenKind::RBracket,
            TokenKind::Ident("int".into()),
            TokenKind::LBrace,
            TokenKind::Int(1),
            TokenKind::RBrace,
            TokenKind::LBracket,
            TokenKind::Int(0),
            TokenKind::RBracket,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_assignment_statements() {
    let tokens = lex("value = 7").expect("assignment should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("value".into()),
            TokenKind::Equal,
            TokenKind::Int(7),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_select_keywords() {
    let tokens = lex("select { case <-values: default: }").expect("select should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Select,
            TokenKind::LBrace,
            TokenKind::Case,
            TokenKind::Arrow,
            TokenKind::Ident("values".into()),
            TokenKind::Colon,
            TokenKind::Default,
            TokenKind::Colon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_return_statements() {
    let tokens = lex("return value").expect("return should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Return,
            TokenKind::Ident("value".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_plus_expressions() {
    let tokens = lex("left + right").expect("plus expression should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("left".into()),
            TokenKind::Plus,
            TokenKind::Ident("right".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_boolean_literals() {
    let tokens = lex("true false").expect("boolean literals should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![TokenKind::True, TokenKind::False, TokenKind::Eof]
    );
}

#[test]
fn lexes_comparison_operators() {
    let tokens = lex("a == b != c < d <= e > f >= g").expect("comparison operators should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("a".into()),
            TokenKind::EqualEqual,
            TokenKind::Ident("b".into()),
            TokenKind::BangEqual,
            TokenKind::Ident("c".into()),
            TokenKind::Less,
            TokenKind::Ident("d".into()),
            TokenKind::LessEqual,
            TokenKind::Ident("e".into()),
            TokenKind::Greater,
            TokenKind::Ident("f".into()),
            TokenKind::GreaterEqual,
            TokenKind::Ident("g".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_if_else_keywords() {
    let tokens = lex("if ready {} else {}").expect("if/else should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::If,
            TokenKind::Ident("ready".into()),
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::Else,
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_unary_and_logical_operators() {
    let tokens = lex("!ready && left || right").expect("operators should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Bang,
            TokenKind::Ident("ready".into()),
            TokenKind::AndAnd,
            TokenKind::Ident("left".into()),
            TokenKind::OrOr,
            TokenKind::Ident("right".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_arithmetic_operators() {
    let tokens = lex("left - right * next / tail << 2 >> 1 | mask & bits ^ flip &^ clear")
        .expect("operators should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("left".into()),
            TokenKind::Minus,
            TokenKind::Ident("right".into()),
            TokenKind::Star,
            TokenKind::Ident("next".into()),
            TokenKind::Slash,
            TokenKind::Ident("tail".into()),
            TokenKind::ShiftLeft,
            TokenKind::Int(2),
            TokenKind::ShiftRight,
            TokenKind::Int(1),
            TokenKind::BitOr,
            TokenKind::Ident("mask".into()),
            TokenKind::BitAnd,
            TokenKind::Ident("bits".into()),
            TokenKind::Caret,
            TokenKind::Ident("flip".into()),
            TokenKind::BitClear,
            TokenKind::Ident("clear".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_increment_and_decrement_operators() {
    let tokens = lex("count++ value--").expect("operators should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("count".into()),
            TokenKind::PlusPlus,
            TokenKind::Ident("value".into()),
            TokenKind::MinusMinus,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_for_keyword() {
    let tokens = lex("for ready {}").expect("for should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::For,
            TokenKind::Ident("ready".into()),
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_break_and_continue_keywords() {
    let tokens = lex("break continue").expect("loop control keywords should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![TokenKind::Break, TokenKind::Continue, TokenKind::Eof]
    );
}

#[test]
fn lexes_go_and_defer_keywords() {
    let tokens = lex("go worker() defer fmt.Println(value)").expect("keywords should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(kinds[0], TokenKind::Go);
    assert!(matches!(kinds[4], TokenKind::Defer));
}

#[test]
fn lexes_switch_case_default_keywords() {
    let tokens = lex("switch x { case 1: default: }").expect("switch keywords should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Switch,
            TokenKind::Ident("x".into()),
            TokenKind::LBrace,
            TokenKind::Case,
            TokenKind::Int(1),
            TokenKind::Colon,
            TokenKind::Default,
            TokenKind::Colon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_type_and_struct_keywords() {
    let tokens = lex("type Point struct {}").expect("type keywords should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Type,
            TokenKind::Ident("Point".into()),
            TokenKind::Struct,
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_interface_keyword() {
    let tokens = lex("type Any interface {}").expect("interface keyword should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Type,
            TokenKind::Ident("Any".into()),
            TokenKind::Interface,
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_chan_keyword() {
    let tokens = lex("var values chan int").expect("chan keyword should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Var,
            TokenKind::Ident("values".into()),
            TokenKind::Chan,
            TokenKind::Ident("int".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_channel_arrow() {
    let tokens = lex("values <- 7 result := <-values").expect("arrow should lex");
    let kinds: Vec<TokenKind> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("values".into()),
            TokenKind::Arrow,
            TokenKind::Int(7),
            TokenKind::Ident("result".into()),
            TokenKind::ColonEqual,
            TokenKind::Arrow,
            TokenKind::Ident("values".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_raw_string_literal() {
    let tokens = lex(r#"x := `hello\nworld`"#).unwrap();
    let kinds: Vec<_> = tokens.iter().map(|t| t.kind.clone()).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("x".into()),
            TokenKind::ColonEqual,
            TokenKind::String(r"hello\nworld".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_raw_string_with_newlines() {
    let tokens = lex("`line1\nline2`").unwrap();
    let kinds: Vec<_> = tokens.iter().map(|t| t.kind.clone()).collect();
    assert_eq!(
        kinds,
        vec![TokenKind::String("line1\nline2".into()), TokenKind::Eof,]
    );
}
