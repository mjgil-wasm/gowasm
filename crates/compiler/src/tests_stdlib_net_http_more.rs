use gowasm_vm::{
    CapabilityRequest, FetchHeader, FetchRequest, FetchResponse, FetchStartRequest, RunOutcome, Vm,
};

use crate::{
    compile_source, tests_stdlib_net_http_support::complete_streamed_fetch_with_buffered_response,
};

#[test]
fn net_http_head_uses_head_fetch_requests() {
    let source = r#"
package main
import "fmt"
import "net/http"

func main() {
    resp, err := http.Head("https://example.com/check?q=1")
    fmt.Println(err == nil, resp != nil)
    fmt.Println(resp.Status, resp.StatusCode, resp.Header.Get("X-Reply"), resp.Body != nil)

    buf := []byte("!!")
    n, readErr := resp.Body.Read(buf)
    fmt.Println(n, readErr != nil)
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
                    method: "HEAD".into(),
                    url: "https://example.com/check?q=1".into(),
                    headers: Vec::new(),
                    body: Vec::new(),
                    context_deadline_unix_millis: None,
                }
            );
        }
        other => panic!("unexpected run outcome: {other:?}"),
    }

    vm.set_fetch_response(FetchResponse {
        status_code: 204,
        status: "204 No Content".into(),
        url: String::new(),
        headers: vec![FetchHeader {
            name: "x-reply".into(),
            values: vec!["ok".into()],
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
        "true true\n204 No Content 204 ok true\n0 true\n"
    );
}

#[test]
fn net_http_default_client_do_works_from_package_values() {
    let source = r#"
package main
import "fmt"
import "net/http"

var defaultClient = http.DefaultClient

func main() {
    req, reqErr := http.NewRequest("DELETE", "https://example.com/default?q=1", nil)
    fmt.Println(reqErr == nil, req != nil, defaultClient != nil)

    req.Header.Set("X-Token", "abc")

    resp, err := defaultClient.Do(req)
    fmt.Println(err == nil, resp != nil)
    fmt.Println(resp.Status, resp.StatusCode, resp.Header.Get("X-Reply"), resp.Body != nil)

    buf := []byte("!!!!")
    n, readErr := resp.Body.Read(buf)
    fmt.Println(n, string(buf[:n]), readErr == nil)
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
                    method: "DELETE".into(),
                    url: "https://example.com/default?q=1".into(),
                    headers: vec![FetchHeader {
                        name: "X-Token".into(),
                        values: vec!["abc".into()],
                    }],
                    body: Vec::new(),
                    context_deadline_unix_millis: None,
                }
            );
        }
        other => panic!("unexpected run outcome: {other:?}"),
    }

    vm.set_fetch_response(FetchResponse {
        status_code: 202,
        status: "202 Accepted".into(),
        url: String::new(),
        headers: vec![FetchHeader {
            name: "x-reply".into(),
            values: vec!["ok".into()],
        }],
        body: b"done".to_vec(),
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
        "true true true\ntrue true\n202 Accepted 202 ok true\n4 done true\n"
    );
}

#[test]
fn net_http_client_get_and_head_use_shared_fetch_requests() {
    let source = r#"
package main
import "fmt"
import "net/http"

func main() {
    client := http.DefaultClient

    getResp, getErr := client.Get("https://example.com/get?q=1")
    fmt.Println(getErr == nil, getResp != nil)
    fmt.Println(getResp.Status, getResp.StatusCode, getResp.Header.Get("X-Reply"), getResp.Body != nil)

    headResp, headErr := client.Head("https://example.com/head?q=1")
    fmt.Println(headErr == nil, headResp != nil)
    fmt.Println(headResp.Status, headResp.StatusCode, headResp.Header.Get("X-Reply"), headResp.Body != nil)

    buf := []byte("!!")
    n, readErr := headResp.Body.Read(buf)
    fmt.Println(n, readErr != nil)
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
                    url: "https://example.com/get?q=1".into(),
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
        url: String::new(),
        headers: vec![FetchHeader {
            name: "x-reply".into(),
            values: vec!["get".into()],
        }],
        body: b"body".to_vec(),
    });

    match vm
        .resume_program(&program)
        .expect("program should pause for the second fetch")
    {
        RunOutcome::CapabilityRequest(CapabilityRequest::Fetch { request }) => {
            assert_eq!(
                request,
                FetchRequest {
                    method: "HEAD".into(),
                    url: "https://example.com/head?q=1".into(),
                    headers: Vec::new(),
                    body: Vec::new(),
                    context_deadline_unix_millis: None,
                }
            );
        }
        other => panic!("unexpected resumed outcome: {other:?}"),
    }

    vm.set_fetch_response(FetchResponse {
        status_code: 204,
        status: "204 No Content".into(),
        url: String::new(),
        headers: vec![FetchHeader {
            name: "x-reply".into(),
            values: vec!["head".into()],
        }],
        body: Vec::new(),
    });

    match vm
        .resume_program(&program)
        .expect("program should complete after second fetch resume")
    {
        RunOutcome::Completed => {}
        other => panic!("unexpected resumed outcome: {other:?}"),
    }

    assert_eq!(
        vm.stdout(),
        "true true\n200 OK 200 get true\ntrue true\n204 No Content 204 head true\n0 true\n"
    );
}

#[test]
fn net_http_client_post_buffers_custom_reader_bodies() {
    let source = r#"
package main
import "errors"
import "fmt"
import "net/http"

type customReader struct {
    data string
    offset int
}

func (r *customReader) Read(p []byte) (int, error) {
    if r.offset >= len(r.data) {
        return 0, errors.New("EOF")
    }

    remaining := []byte(r.data[r.offset:])
    if len(remaining) > 2 {
        remaining = remaining[:2]
    }

    n := copy(p, remaining)
    r.offset += n
    if r.offset >= len(r.data) {
        return n, errors.New("EOF")
    }
    return n, nil
}

func main() {
    client := http.DefaultClient
    reader := &customReader{data: "chunked"}
    resp, err := client.Post("https://example.com/upload", "text/plain", reader)
    fmt.Println(err == nil, resp != nil, reader.offset)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.enable_capability_requests();

    complete_streamed_fetch_with_buffered_response(
        &mut vm,
        &program,
        FetchStartRequest {
            session_id: 0,
            method: "POST".into(),
            url: "https://example.com/upload".into(),
            headers: vec![FetchHeader {
                name: "Content-Type".into(),
                values: vec!["text/plain".into()],
            }],
            context_deadline_unix_millis: None,
        },
        &[b"ch", b"un", b"ke", b"d"],
        FetchResponse {
            status_code: 204,
            status: "204 No Content".into(),
            url: String::new(),
            headers: Vec::new(),
            body: Vec::new(),
        },
    );

    assert_eq!(vm.stdout(), "true true 7\n");
}

#[test]
fn net_http_post_form_helpers_use_encoded_form_bodies() {
    let source = r#"
package main
import "fmt"
import "net/http"
import "net/url"

func main() {
    values := make(url.Values)
    values["b"] = []string{"two words"}
    values["a"] = []string{"1", "2"}

    resp, err := http.PostForm("https://example.com/form", values)
    fmt.Println(err == nil, resp != nil)

    client := http.DefaultClient
    clientResp, clientErr := client.PostForm("https://example.com/client", values)
    fmt.Println(clientErr == nil, clientResp != nil)
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
                    method: "POST".into(),
                    url: "https://example.com/form".into(),
                    headers: vec![FetchHeader {
                        name: "Content-Type".into(),
                        values: vec!["application/x-www-form-urlencoded".into()],
                    }],
                    body: b"a=1&a=2&b=two+words".to_vec(),
                    context_deadline_unix_millis: None,
                }
            );
        }
        other => panic!("unexpected run outcome: {other:?}"),
    }

    vm.set_fetch_response(FetchResponse {
        status_code: 200,
        status: "200 OK".into(),
        url: String::new(),
        headers: Vec::new(),
        body: Vec::new(),
    });

    match vm
        .resume_program(&program)
        .expect("program should pause for the second fetch")
    {
        RunOutcome::CapabilityRequest(CapabilityRequest::Fetch { request }) => {
            assert_eq!(
                request,
                FetchRequest {
                    method: "POST".into(),
                    url: "https://example.com/client".into(),
                    headers: vec![FetchHeader {
                        name: "Content-Type".into(),
                        values: vec!["application/x-www-form-urlencoded".into()],
                    }],
                    body: b"a=1&a=2&b=two+words".to_vec(),
                    context_deadline_unix_millis: None,
                }
            );
        }
        other => panic!("unexpected resumed outcome: {other:?}"),
    }

    vm.set_fetch_response(FetchResponse {
        status_code: 204,
        status: "204 No Content".into(),
        url: String::new(),
        headers: Vec::new(),
        body: Vec::new(),
    });

    match vm
        .resume_program(&program)
        .expect("program should complete after second fetch resume")
    {
        RunOutcome::Completed => {}
        other => panic!("unexpected resumed outcome: {other:?}"),
    }

    assert_eq!(vm.stdout(), "true true\ntrue true\n");
}

#[test]
fn net_http_package_helpers_keep_their_own_missing_capability_errors() {
    let source = r#"
package main
import "fmt"
import "net/http"
import "net/url"

func main() {
    getResp, getErr := http.Get("https://example.com/get")
    headResp, headErr := http.Head("https://example.com/head")
    postResp, postErr := http.Post("https://example.com/post", "text/plain", nil)
    formResp, formErr := http.PostForm("https://example.com/form", make(url.Values))

    fmt.Println(getResp == nil, getErr)
    fmt.Println(headResp == nil, headErr)
    fmt.Println(postResp == nil, postErr)
    fmt.Println(formResp == nil, formErr)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();

    match vm
        .start_program(&program)
        .expect("program should complete without fetch capabilities")
    {
        RunOutcome::Completed => {}
        other => panic!("unexpected run outcome: {other:?}"),
    }

    assert_eq!(
        vm.stdout(),
        "true net/http: http.Get requires a host-provided fetch capability\n\
true net/http: http.Head requires a host-provided fetch capability\n\
true net/http: http.Post requires a host-provided fetch capability\n\
true net/http: http.PostForm requires a host-provided fetch capability\n"
    );
}

#[test]
fn net_http_client_helpers_keep_their_own_missing_capability_errors() {
    let source = r#"
package main
import "fmt"
import "net/http"
import "net/url"

func main() {
    client := http.DefaultClient

    getResp, getErr := client.Get("https://example.com/get")
    headResp, headErr := client.Head("https://example.com/head")
    postResp, postErr := client.Post("https://example.com/post", "text/plain", nil)
    formResp, formErr := client.PostForm("https://example.com/form", make(url.Values))

    fmt.Println(getResp == nil, getErr)
    fmt.Println(headResp == nil, headErr)
    fmt.Println(postResp == nil, postErr)
    fmt.Println(formResp == nil, formErr)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();

    match vm
        .start_program(&program)
        .expect("program should complete without fetch capabilities")
    {
        RunOutcome::Completed => {}
        other => panic!("unexpected run outcome: {other:?}"),
    }

    assert_eq!(
        vm.stdout(),
        "true net/http: (*http.Client).Get requires a host-provided fetch capability\n\
true net/http: (*http.Client).Head requires a host-provided fetch capability\n\
true net/http: (*http.Client).Post requires a host-provided fetch capability\n\
true net/http: (*http.Client).PostForm requires a host-provided fetch capability\n"
    );
}

#[test]
fn net_http_shared_request_builder_error_paths() {
    let source = r#"
package main
import "fmt"
import "net/http"

func main() {
    client := http.DefaultClient

    req, _ := http.NewRequest("GET", "https://example.com/path", nil)
    req.Method = "bad method"
    resp, err := client.Do(req)
    fmt.Println(resp == nil, err)

    req, _ = http.NewRequest("GET", "https://example.com/path", nil)
    req.URL = nil
    resp, err = client.Do(req)
    fmt.Println(resp == nil, err)

    getResp, getErr := http.Get("bad\nurl")
    headResp, headErr := http.Head("bad\nurl")
    postResp, postErr := http.Post("bad\nurl", "text/plain", nil)
    fmt.Println(getResp == nil, getErr)
    fmt.Println(headResp == nil, headErr)
    fmt.Println(postResp == nil, postErr)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");

    assert_eq!(
        vm.stdout(),
        "true net/http: invalid method \"bad method\"\n\
true net/http: nil Request.URL\n\
true parse \"bad\\nurl\": net/url: invalid control character in URL\n\
true parse \"bad\\nurl\": net/url: invalid control character in URL\n\
true parse \"bad\\nurl\": net/url: invalid control character in URL\n"
    );
}

#[test]
fn net_http_fetch_resume_errors_surface_as_net_http_errors() {
    let source = r#"
package main
import "fmt"
import "net/http"

func main() {
    resp, err := http.Get("https://example.com/unreachable")
    fmt.Println(resp == nil, err)
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
                    url: "https://example.com/unreachable".into(),
                    headers: Vec::new(),
                    body: Vec::new(),
                    context_deadline_unix_millis: None,
                }
            );
        }
        other => panic!("unexpected run outcome: {other:?}"),
    }

    vm.set_fetch_error("fetch failed for GET https://example.com/unreachable: network down");

    match vm
        .resume_program(&program)
        .expect("program should complete after fetch error resume")
    {
        RunOutcome::Completed => {}
        other => panic!("unexpected resumed outcome: {other:?}"),
    }

    assert_eq!(
        vm.stdout(),
        "true net/http: fetch failed for GET https://example.com/unreachable: network down\n"
    );
}

#[test]
fn net_http_fetch_responses_normalize_empty_status_text() {
    let source = r#"
package main
import "fmt"
import "net/http"

func main() {
    resp, err := http.Get("https://example.com/status")
    fmt.Println(err == nil, resp.Status, resp.StatusCode)
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
                    url: "https://example.com/status".into(),
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
        status: String::new(),
        url: String::new(),
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

    assert_eq!(vm.stdout(), "true 200 OK 200\n");
}

#[test]
fn net_http_client_do_returns_canceled_request_context_errors_before_fetch() {
    let source = r#"
package main
import "context"
import "fmt"
import "net/http"

func main() {
    ctx, cancel := context.WithCancel(context.Background())
    cancel()

    req, reqErr := http.NewRequestWithContext(ctx, "GET", "https://example.com/cancelled", nil)
    fmt.Println(reqErr == nil, req != nil)

    resp, err := http.DefaultClient.Do(req)
    fmt.Println(resp == nil, err == context.Canceled, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.enable_capability_requests();

    match vm
        .start_program(&program)
        .expect("program should complete without fetch")
    {
        RunOutcome::Completed => {}
        other => panic!("unexpected run outcome: {other:?}"),
    }

    assert_eq!(vm.stdout(), "true true\ntrue true context canceled\n");
}

#[test]
fn net_http_client_do_returns_deadline_exceeded_request_context_errors_before_fetch() {
    let source = r#"
package main
import "context"
import "fmt"
import "net/http"
import "time"

func main() {
    ctx, cancel := context.WithDeadline(context.Background(), time.UnixMilli(10))
    defer cancel()

    req, reqErr := http.NewRequestWithContext(ctx, "GET", "https://example.com/expired", nil)
    fmt.Println(reqErr == nil, req != nil)

    resp, err := http.DefaultClient.Do(req)
    fmt.Println(resp == nil, err == context.DeadlineExceeded, err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.set_time_now_override_unix_nanos(20_000_000);
    vm.enable_capability_requests();

    match vm
        .start_program(&program)
        .expect("program should complete without fetch")
    {
        RunOutcome::Completed => {}
        other => panic!("unexpected run outcome: {other:?}"),
    }

    assert_eq!(
        vm.stdout(),
        "true true\ntrue true context deadline exceeded\n"
    );
}
