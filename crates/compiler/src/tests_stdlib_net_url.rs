use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_net_url_parse() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    parsed, err := url.Parse("https://example.com/submit?q=1#frag")
    fmt.Println(err == nil, parsed != nil)
    fmt.Println(parsed.Scheme, parsed.Host, parsed.Path, parsed.RawQuery, parsed.Fragment)
    fmt.Println(parsed.String())

    parsed, err = url.Parse("/api/v1/users?active=1")
    fmt.Println(err == nil, parsed.Scheme == "", parsed.Host == "", parsed.Path, parsed.RawQuery)
    fmt.Println(parsed.String())

    parsed, err = url.Parse("mailto:dev@example.com")
    fmt.Println(err == nil, parsed.Scheme, parsed.Opaque, parsed.Path == "")
    fmt.Println(parsed.String())

    parsed, err = url.Parse("mailto:dev@example.com?")
    fmt.Println(err == nil, parsed.Scheme, parsed.Opaque, parsed.ForceQuery, parsed.RawQuery == "")
    fmt.Println(parsed.String())

    parsed, err = url.Parse("//cdn.example.com/assets/app.js#hash")
    fmt.Println(err == nil, parsed.Scheme == "", parsed.Host, parsed.Path, parsed.Fragment)
    fmt.Println(parsed.String())

    parsed, err = url.Parse("bad\nurl")
    fmt.Println(err != nil, parsed == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true true\nhttps example.com /submit q=1 frag\nhttps://example.com/submit?q=1#frag\ntrue true true /api/v1/users active=1\n/api/v1/users?active=1\ntrue mailto dev@example.com true\nmailto:dev@example.com\ntrue mailto dev@example.com true true\nmailto:dev@example.com?\ntrue true cdn.example.com /assets/app.js hash\n//cdn.example.com/assets/app.js#hash\ntrue true\n"
    );
}

#[test]
fn compiles_and_runs_net_url_parse_request_uri() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    parsed, err := url.ParseRequestURI("/api/v1/users?active=1")
    fmt.Println(err == nil, parsed.Path, parsed.RawQuery, parsed.RequestURI())

    parsed, err = url.ParseRequestURI("https://example.com/a%2Fb?q=1")
    fmt.Println(err == nil, parsed.Scheme, parsed.Host, parsed.Path, parsed.RawPath, parsed.RawQuery)
    fmt.Println(parsed.String())

    parsed, err = url.ParseRequestURI("mailto:dev@example.com?")
    fmt.Println(err == nil, parsed.Opaque, parsed.ForceQuery, parsed.RequestURI())

    parsed, err = url.ParseRequestURI("*")
    fmt.Println(err == nil, parsed.Path, parsed.RequestURI())

    parsed, err = url.ParseRequestURI("//cdn.example.com/assets/app.js")
    fmt.Println(err == nil, parsed.Host == "", parsed.Path)

    parsed, err = url.ParseRequestURI("")
    fmt.Println(err != nil, parsed == nil)

    parsed, err = url.ParseRequestURI("relative/path")
    fmt.Println(err != nil, parsed == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true /api/v1/users active=1 /api/v1/users?active=1\ntrue https example.com /a/b /a%2Fb q=1\nhttps://example.com/a%2Fb?q=1\ntrue dev@example.com true dev@example.com?\ntrue * *\ntrue true //cdn.example.com/assets/app.js\ntrue true\ntrue true\n"
    );
}

#[test]
fn compiles_and_runs_net_url_is_abs() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    parsed, err := url.Parse("/api/v1/users?active=1")
    fmt.Println(err == nil, parsed.IsAbs())

    parsed, err = url.Parse("https://example.com/a%2Fb?q=1")
    fmt.Println(err == nil, parsed.IsAbs())

    parsed, err = url.Parse("mailto:dev@example.com?")
    fmt.Println(err == nil, parsed.IsAbs())

    var zero url.URL
    fmt.Println(zero.IsAbs())

    zero.Scheme = "custom"
    fmt.Println(zero.IsAbs())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true false\ntrue true\ntrue true\nfalse\ntrue\n"
    );
}

#[test]
fn compiles_and_runs_net_url_host_accessors() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    parsed, err := url.Parse("https://example.com:8080/path")
    fmt.Println(err == nil, parsed.Hostname(), parsed.Port())

    parsed, err = url.Parse("https://[2001:db8::1]:443/path")
    fmt.Println(err == nil, parsed.Hostname(), parsed.Port())

    parsed, err = url.Parse("https://example.com:/path")
    fmt.Println(err == nil, parsed.Hostname(), parsed.Port() == "")

    parsed, err = url.Parse("https://example.com:notaport/path")
    fmt.Println(err == nil, parsed.Hostname(), parsed.Port() == "")

    var manual url.URL
    manual.Host = "[2001:db8::2]"
    fmt.Println(manual.Hostname(), manual.Port() == "")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true example.com 8080\ntrue 2001:db8::1 443\ntrue example.com true\ntrue example.com:notaport true\n2001:db8::2 true\n"
    );
}

#[test]
fn compiles_and_runs_net_url_parse_raw_path_and_fragment_hints() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    parsed, err := url.Parse("https://example.com/a%2Fb?q=1#frag%2Fpart")
    fmt.Println(err == nil, parsed != nil)
    fmt.Println(parsed.Path, parsed.RawPath, parsed.RawQuery, parsed.Fragment, parsed.RawFragment)
    fmt.Println(parsed.EscapedPath())
    fmt.Println(parsed.EscapedFragment())
    fmt.Println(parsed.String())

    parsed.Path = "/space path"
    parsed.RawPath = "/broken%zz"
    parsed.Fragment = "frag ment"
    parsed.RawFragment = "%zz"
    fmt.Println(parsed.EscapedPath())
    fmt.Println(parsed.EscapedFragment())
    fmt.Println(parsed.String())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true true\n/a/b /a%2Fb q=1 frag/part frag%2Fpart\n/a%2Fb\nfrag%2Fpart\nhttps://example.com/a%2Fb?q=1#frag%2Fpart\n/space%20path\nfrag%20ment\nhttps://example.com/space%20path?q=1#frag%20ment\n"
    );
}

#[test]
fn compiles_and_runs_net_url_request_uri() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    parsed, err := url.Parse("https://example.com/a%2Fb?q=1#frag%2Fpart")
    fmt.Println(err == nil, parsed.RequestURI())

    parsed.Path = "/space path"
    parsed.RawPath = "/broken%zz"
    fmt.Println(parsed.RequestURI())

    parsed.Path = ""
    parsed.RawPath = ""
    parsed.RawQuery = "x=1"
    fmt.Println(parsed.RequestURI())

    parsed, err = url.Parse("mailto:dev@example.com?")
    fmt.Println(err == nil, parsed.RequestURI())

    var opaque url.URL
    opaque.Scheme = "custom"
    opaque.Opaque = "//example.com/path"
    opaque.RawQuery = "x=1"
    fmt.Println(opaque.RequestURI())

    var zero url.URL
    fmt.Println(zero.RequestURI())

    zero.ForceQuery = true
    fmt.Println(zero.RequestURI())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true /a%2Fb?q=1\n/space%20path?q=1\n/?x=1\ntrue dev@example.com?\ncustom://example.com/path?x=1\n/\n/?\n"
    );
}

#[test]
fn compiles_and_runs_net_url_string_with_opaque_and_force_query() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    parsed, err := url.Parse("mailto:dev@example.com?")
    fmt.Println(err == nil, parsed.String(), parsed.RequestURI())

    var forced url.URL
    forced.Scheme = "https"
    forced.Host = "example.com"
    forced.ForceQuery = true
    fmt.Println(forced.String(), forced.RequestURI())

    var opaque url.URL
    opaque.Scheme = "custom"
    opaque.Opaque = "//example.com/path"
    opaque.RawQuery = "x=1"
    opaque.Fragment = "frag ment"
    fmt.Println(opaque.String())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true mailto:dev@example.com? dev@example.com?\nhttps://example.com? /?\ncustom://example.com/path?x=1#frag%20ment\n"
    );
}

#[test]
fn compiles_and_runs_net_url_values_encode() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    var nilValues url.Values
    fmt.Println(nilValues.Encode() == "")

    values := make(url.Values)
    values["z"] = []string{"last"}
    values["a"] = []string{"1", "two words"}
    values["space key"] = []string{"x+y", ""}
    values["slash"] = []string{"a/b"}
    values["skip"] = []string{}

    fmt.Println(values.Encode())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true\na=1&a=two+words&slash=a%2Fb&space+key=x%2By&space+key=&z=last\n"
    );
}

#[test]
fn compiles_and_runs_net_url_values_get_set_and_add() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    var nilValues url.Values
    fmt.Println(nilValues.Get("missing") == "")

    values := make(url.Values)
    fmt.Println(values.Get("missing") == "", values.Encode() == "")

    values.Add("q", "go wasm")
    values.Add("q", "codex")
    values.Set("empty", "")
    fmt.Println(values.Get("q"))
    fmt.Println(values.Encode())

    values.Set("q", "reset")
    fmt.Println(values.Get("q"))
    fmt.Println(values.Encode())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true\ntrue true\ngo wasm\nempty=&q=go+wasm&q=codex\nreset\nempty=&q=reset\n"
    );
}

#[test]
fn compiles_and_runs_net_url_values_del() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    var nilValues url.Values
    nilValues.Del("missing")
    fmt.Println(nilValues == nil)

    values := make(url.Values)
    values.Add("q", "go wasm")
    values.Add("q", "codex")
    values.Set("empty", "")
    values.Del("missing")
    fmt.Println(values.Encode())

    values.Del("q")
    fmt.Println(values.Get("q") == "", values.Encode())

    values.Del("empty")
    fmt.Println(values.Encode() == "")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true\nempty=&q=go+wasm&q=codex\ntrue empty=\ntrue\n"
    );
}

#[test]
fn compiles_and_runs_net_url_values_has() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    var nilValues url.Values
    fmt.Println(nilValues.Has("missing"))

    values := make(url.Values)
    fmt.Println(values.Has("missing"))

    values.Add("q", "go wasm")
    values["empty"] = []string{}
    fmt.Println(values.Has("q"), values.Has("empty"), values.Has("missing"))

    values.Del("q")
    fmt.Println(values.Has("q"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false\nfalse\ntrue true false\nfalse\n");
}

#[test]
fn compiles_and_runs_net_url_marshal_binary() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    parsed, err := url.Parse("https://example.com/a%2Fb?q=1#frag%2Fpart")
    fmt.Println(err == nil)

    text, err := parsed.MarshalBinary()
    fmt.Println(err == nil, string(text))

    var manual url.URL
    manual.Scheme = "custom"
    manual.Opaque = "//example.com/path"
    manual.RawQuery = "x=1"
    text, err = manual.MarshalBinary()
    fmt.Println(err == nil, string(text))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true\ntrue https://example.com/a%2Fb?q=1#frag%2Fpart\ntrue custom://example.com/path?x=1\n"
    );
}

#[test]
fn compiles_and_runs_net_url_unmarshal_binary() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    var parsed url.URL
    err := parsed.UnmarshalBinary([]byte("https://example.com/a%2Fb?q=1#frag%2Fpart"))
    fmt.Println(err == nil, parsed.String(), parsed.Hostname(), parsed.Port() == "")

    err = parsed.UnmarshalBinary([]byte("bad\nurl"))
    fmt.Println(err != nil, parsed.String())

    var opaque url.URL
    err = opaque.UnmarshalBinary([]byte("mailto:dev@example.com?"))
    fmt.Println(err == nil, opaque.String(), opaque.RequestURI())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true https://example.com/a%2Fb?q=1#frag%2Fpart example.com true\ntrue https://example.com/a%2Fb?q=1#frag%2Fpart\ntrue mailto:dev@example.com? dev@example.com?\n"
    );
}

#[test]
fn compiles_and_runs_net_url_redacted() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    parsed, err := url.Parse("https://example.com/a%2Fb?q=1#frag%2Fpart")
    fmt.Println(err == nil, parsed.Redacted())

    var nilURL *url.URL
    fmt.Println(nilURL.Redacted() == "")

    var opaque url.URL
    opaque.Scheme = "mailto"
    opaque.Opaque = "dev@example.com"
    opaque.ForceQuery = true
    fmt.Println(opaque.Redacted())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true https://example.com/a%2Fb?q=1#frag%2Fpart\ntrue\nmailto:dev@example.com?\n"
    );
}

#[test]
fn compiles_and_runs_net_url_resolve_reference() {
    let source = r##"
package main
import "fmt"
import "net/url"

func main() {
    base, baseErr := url.Parse("https://example.com/a/b/c?q=1#base")
    fmt.Println(baseErr == nil)

    ref, refParseErr := url.Parse("../d/e?x=2")
    fmt.Println(refParseErr == nil, base.ResolveReference(ref).String())

    abs, absErr := url.Parse("https://cdn.example.com//asset/./v1")
    fmt.Println(absErr == nil, base.ResolveReference(abs).String())

    frag, fragErr := url.Parse("#next")
    fmt.Println(fragErr == nil, base.ResolveReference(frag).String())

    opaqueBase, opaqueErr := url.Parse("mailto:dev@example.com?")
    emptyRef, refErr := url.Parse("")
    fmt.Println(opaqueErr == nil, refErr == nil, opaqueBase.ResolveReference(emptyRef).String())
}
"##;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true\ntrue https://example.com/a/d/e?x=2\ntrue https://cdn.example.com//asset/v1\ntrue https://example.com/a/b/c?q=1#next\ntrue true mailto:dev@example.com\n"
    );
}

#[test]
fn compiles_and_runs_net_url_parse_method() {
    let source = r##"
package main
import "fmt"
import "net/url"

func main() {
    base, baseErr := url.Parse("https://example.com/a/b/c?q=1#base")
    fmt.Println(baseErr == nil)

    var resolved *url.URL
    var err error

    resolved, err = base.Parse("../d/e?x=2")
    fmt.Println(err == nil, resolved.String())

    resolved, err = base.Parse("https://cdn.example.com//asset/./v1")
    fmt.Println(err == nil, resolved.String())

    resolved, err = base.Parse("#next")
    fmt.Println(err == nil, resolved.String())

    _, err = base.Parse("bad\nurl")
    fmt.Println(err != nil)

    opaqueBase, opaqueErr := url.Parse("mailto:dev@example.com?")
    resolved, err = opaqueBase.Parse("")
    fmt.Println(opaqueErr == nil, err == nil, resolved.String())
}
"##;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true\ntrue https://example.com/a/d/e?x=2\ntrue https://cdn.example.com//asset/v1\ntrue https://example.com/a/b/c?q=1#next\ntrue\ntrue true mailto:dev@example.com\n"
    );
}

#[test]
fn compiles_and_runs_net_url_join_path_helpers() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    base, err := url.Parse("https://example.com/a%2Fb?q=1#frag")
    fmt.Println(err == nil)

    var joined *url.URL
    joined = base.JoinPath("c d", "e%2Ff")
    fmt.Println(joined.String())
    fmt.Println(joined.Path, joined.RawPath, joined.RawQuery, joined.Fragment)
    fmt.Println(base.String())

    var relative url.URL
    relative.Path = "a/b"
    fmt.Println(relative.JoinPath("..", "c").String())

    opaque, opaqueErr := url.Parse("mailto:dev@example.com")
    joined = opaque.JoinPath("x")
    fmt.Println(opaqueErr == nil, joined.String(), joined.Path, joined.RawPath == "")

    joined = base.JoinPath("%zz")
    fmt.Println(joined.String())

    result, joinErr := url.JoinPath("https://example.com/base/", "x y", "/z/")
    fmt.Println(joinErr == nil, result)

    result, joinErr = url.JoinPath("https://example.com/a/../b//")
    fmt.Println(joinErr == nil, result)

    result, joinErr = url.JoinPath("https://example.com/a", "%zz")
    fmt.Println(joinErr == nil, result)

    _, joinErr = url.JoinPath("bad\nurl", "x")
    fmt.Println(joinErr != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true\nhttps://example.com/a/b/c%20d/e/f?q=1#frag\n/a/b/c d/e/f /a%2Fb/c d/e%2Ff q=1 frag\nhttps://example.com/a%2Fb?q=1#frag\na/c\ntrue mailto:dev@example.com x true\nhttps://example.com/a%2Fb?q=1#frag\ntrue https://example.com/base/x%20y/z/\ntrue https://example.com/b/\ntrue https://example.com/a\ntrue\n"
    );
}

#[test]
fn net_url_values_set_on_nil_map_fails_at_runtime() {
    let source = r#"
package main
import "net/url"

func main() {
    var values url.Values
    values.Set("q", "x")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("nil url.Values writes should fail");
    assert!(error.to_string().contains("cannot assign into a nil map"));
}

#[test]
fn compiles_and_runs_net_url_query_helpers() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    fmt.Println(url.QueryEscape("space key/x+y?=z&"))

    values, err := url.ParseQuery("z=last&a=1&a=two+words&flag&space+key=x%2By&slash=a%2Fb")
    fmt.Println(err == nil, values.Encode())

    values, err = url.ParseQuery("ok=1&bad=%zz&tail=2")
    fmt.Println(err != nil, values.Encode(), err)

    values, err = url.ParseQuery("semi=1;tail=2")
    fmt.Println(err != nil, values.Encode(), err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "space+key%2Fx%2By%3F%3Dz%26\ntrue a=1&a=two+words&flag=&slash=a%2Fb&space+key=x%2By&z=last\ntrue ok=1&tail=2 invalid URL escape \"%zz\"\ntrue  invalid semicolon separator in query\n"
    );
}

#[test]
fn compiles_and_runs_net_url_path_helpers() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    fmt.Println(url.PathEscape("my/cool+blog&about,stuff"))

    decoded, err := url.PathUnescape("my%2Fcool+blog&about%2Cstuff")
    fmt.Println(err == nil, decoded)

    decoded, err = url.PathUnescape("space%20path")
    fmt.Println(err == nil, decoded)

    decoded, err = url.PathUnescape("%zz")
    fmt.Println(err != nil, decoded == "")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "my%2Fcool+blog&about%2Cstuff\ntrue my/cool+blog&about,stuff\ntrue space path\ntrue true\n"
    );
}

#[test]
fn compiles_and_runs_net_url_escaped_path() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    parsed, err := url.Parse("https://example.com/a b/c")
    fmt.Println(err == nil, parsed != nil)
    fmt.Println(parsed.EscapedPath())

    parsed.Path = "/plus+space path/segment;comma,question?"
    fmt.Println(parsed.EscapedPath())

    parsed.Path = "*"
    fmt.Println(parsed.EscapedPath())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true true\n/a%20b/c\n/plus+space%20path/segment;comma,question%3F\n*\n"
    );
}

#[test]
fn compiles_and_runs_net_url_escaped_fragment() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    parsed, err := url.Parse("https://example.com/path#frag ment")
    fmt.Println(err == nil, parsed != nil)
    fmt.Println(parsed.EscapedFragment())

    parsed.Fragment = "plus+space fragment?and/slash"
    fmt.Println(parsed.EscapedFragment())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true true\nfrag%20ment\nplus+space%20fragment?and/slash\n"
    );
}

#[test]
fn compiles_and_runs_net_url_query_unescape_and_url_query() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    decoded, err := url.QueryUnescape("space+key%2Fx%2By%3F%3Dz%26")
    fmt.Println(err == nil, decoded)

    decoded, err = url.QueryUnescape("%zz")
    fmt.Println(err != nil, decoded == "")

    parsed, parseErr := url.Parse("https://example.com/search?a=1&a=two+words&bad=%zz&tail=2")
    fmt.Println(parseErr == nil, parsed != nil)
    fmt.Println(parsed.Query().Encode())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true space key/x+y?=z&\ntrue true\ntrue true\na=1&a=two+words&tail=2\n"
    );
}
