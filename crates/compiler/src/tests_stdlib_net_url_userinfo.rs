use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn compiles_and_runs_net_url_userinfo_support() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    parsed, err := url.Parse("https://alice%40example.com:p%40ss%3A%2F%3F@example.com/path")
    fmt.Println(err == nil, parsed.User != nil)
    fmt.Println(parsed.User.Username())
    password, ok := parsed.User.Password()
    fmt.Println(password, ok)
    fmt.Println(parsed.String())
    fmt.Println(parsed.Redacted())

    var nilInfo *url.Userinfo
    fmt.Println(nilInfo.Username() == "", nilInfo.String() == "")
    password, ok = nilInfo.Password()
    fmt.Println(password == "", ok)

    parsed.User = url.User("bob@example.com")
    fmt.Println(parsed.String())
    fmt.Println(parsed.User.String())

    parsed.User = url.UserPassword("bob@example.com", "s3cret:/?")
    fmt.Println(parsed.String())
    fmt.Println(parsed.Redacted())
    password, ok = parsed.User.Password()
    fmt.Println(password, ok)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true true\nalice@example.com\np@ss:/? true\nhttps://alice%40example.com:p%40ss%3A%2F%3F@example.com/path\nhttps://alice%40example.com:xxxxx@example.com/path\ntrue true\ntrue false\nhttps://bob%40example.com@example.com/path\nbob%40example.com\nhttps://bob%40example.com:s3cret%3A%2F%3F@example.com/path\nhttps://bob%40example.com:xxxxx@example.com/path\ns3cret:/? true\n"
    );
}

#[test]
fn compiles_and_runs_net_url_userinfo_reference_resolution() {
    let source = r#"
package main
import "fmt"
import "net/url"

func main() {
    base, err := url.Parse("https://alice:secret@example.com/a/b/c")
    fmt.Println(err == nil)

    ref, err := url.Parse("../next?q=1")
    fmt.Println(err == nil, base.ResolveReference(ref).String())

    absolute, err := url.Parse("//cdn.example.com/asset")
    fmt.Println(err == nil, base.ResolveReference(absolute).String())

    opaqueBase, err := url.Parse("mailto:alice@example.com?")
    opaqueRef, err2 := url.Parse("")
    fmt.Println(err == nil, err2 == nil, opaqueBase.ResolveReference(opaqueRef).String())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true\ntrue https://alice:secret@example.com/a/next?q=1\ntrue https://cdn.example.com/asset\ntrue true mailto:alice@example.com\n"
    );
}
