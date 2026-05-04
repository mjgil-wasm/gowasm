use gowasm_vm::{CapabilityRequest, FetchHeader, FetchRequest, FetchResponse, RunOutcome, Vm};

use crate::compile_source;

#[test]
fn net_http_response_location_resolves_relative_redirects_from_final_fetch_urls() {
    let source = r#"
package main
import "fmt"
import "net/http"

func main() {
    resp, err := http.Get("https://example.com/start")
    fmt.Println(err == nil, resp != nil)

    loc, locErr := resp.Location()
    fmt.Println(locErr == nil, loc.String(), loc.Path, loc.RawQuery, loc.Fragment)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.enable_capability_requests();

    match vm
        .start_program(&program)
        .expect("program should pause for fetch")
    {
        RunOutcome::CapabilityRequest(CapabilityRequest::Fetch { request }) => {
            assert_eq!(
                request,
                FetchRequest {
                    method: "GET".into(),
                    url: "https://example.com/start".into(),
                    headers: Vec::new(),
                    body: Vec::new(),
                    context_deadline_unix_millis: None,
                }
            );
        }
        other => panic!("unexpected run outcome: {other:?}"),
    }

    vm.set_fetch_response(FetchResponse {
        status_code: 302,
        status: "302 Found".into(),
        url: "https://example.com/base/dir/index".into(),
        headers: vec![FetchHeader {
            name: "Location".into(),
            values: vec!["../next?q=1#frag".into()],
        }],
        body: Vec::new(),
    });

    match vm
        .resume_program(&program)
        .expect("program should complete after fetch resume")
    {
        RunOutcome::Completed => {}
        other => panic!("unexpected resumed outcome: {other:?}"),
    }

    assert_eq!(
        vm.stdout(),
        "true true\ntrue https://example.com/base/next?q=1#frag /base/next q=1 frag\n"
    );
}

#[test]
fn net_http_response_location_returns_err_no_location_when_header_is_missing() {
    let source = r#"
package main
import "fmt"
import "net/http"

func main() {
    resp, _ := http.Get("https://example.com/start")
    loc, err := resp.Location()
    fmt.Println(loc == nil, err == http.ErrNoLocation, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.enable_capability_requests();

    match vm
        .start_program(&program)
        .expect("program should pause for fetch")
    {
        RunOutcome::CapabilityRequest(CapabilityRequest::Fetch { request }) => {
            assert_eq!(
                request,
                FetchRequest {
                    method: "GET".into(),
                    url: "https://example.com/start".into(),
                    headers: Vec::new(),
                    body: Vec::new(),
                    context_deadline_unix_millis: None,
                }
            );
        }
        other => panic!("unexpected run outcome: {other:?}"),
    }

    vm.set_fetch_response(FetchResponse {
        status_code: 200,
        status: "200 OK".into(),
        url: "https://example.com/start".into(),
        headers: Vec::new(),
        body: Vec::new(),
    });

    match vm
        .resume_program(&program)
        .expect("program should complete after fetch resume")
    {
        RunOutcome::Completed => {}
        other => panic!("unexpected resumed outcome: {other:?}"),
    }

    assert_eq!(
        vm.stdout(),
        "true true http: no Location header in response\n"
    );
}
