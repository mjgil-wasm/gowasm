use super::{
    compile_source, tests_stdlib_net_http_support::complete_streamed_fetch_with_buffered_response,
};
use gowasm_vm::{
    CapabilityRequest, FetchHeader, FetchRequest, FetchResponse, FetchStartRequest, RunOutcome, Vm,
};

#[test]
fn compiles_and_runs_net_http_helpers() {
    let source = r#"
package main
import "fmt"
import "net/http"

func main() {
    canonical := http.CanonicalHeaderKey
    fmt.Println(http.MethodGet, http.MethodPatch, http.MethodTrace)
    fmt.Println(http.TimeFormat)
    fmt.Println(http.TrailerPrefix, http.DefaultMaxHeaderBytes)
    fmt.Println(http.ErrMissingFile)
    fmt.Println(http.ErrNoCookie)
    fmt.Println(http.ErrNoLocation)
    fmt.Println(http.ErrUseLastResponse)
    fmt.Println(http.ErrAbortHandler)
    fmt.Println(http.ErrServerClosed)
    fmt.Println(http.StatusOK, http.StatusTeapot, http.StatusTooManyRequests)
    fmt.Println(canonical("accept-encoding"))
    fmt.Println(canonical("uSER-aGENT"))
    fmt.Println(canonical("foo-bar_baz"))
    fmt.Println(canonical("foo bar"))
    fmt.Println(http.StatusText(http.StatusOK))
    fmt.Println(http.StatusText(http.StatusTeapot))
    fmt.Printf("[%s]\n", http.StatusText(999))
    major, minor, ok := http.ParseHTTPVersion("HTTP/1.1")
    fmt.Println(major, minor, ok)
    major, minor, ok = http.ParseHTTPVersion("HTTP/2")
    fmt.Println(major, minor, ok)
    fmt.Println(http.DetectContentType([]byte{}))
    fmt.Println(http.DetectContentType([]byte("   <!DOCTYPE HTML>...")))
    fmt.Println(http.DetectContentType([]byte("\n<?xml!")))
    fmt.Println(http.DetectContentType([]byte{0x89, 'P', 'N', 'G', '\r', '\n', 0x1A, '\n'}))
    fmt.Println(http.DetectContentType([]byte{0x00, 0x61, 0x73, 0x6d, 0x01, 0x00}))
    fmt.Println(http.DetectContentType([]byte{1, 2, 3}))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "GET PATCH TRACE\nMon, 02 Jan 2006 15:04:05 GMT\nTrailer: 1048576\nhttp: no such file\nhttp: named cookie not present\nhttp: no Location header in response\nnet/http: use last response\nnet/http: abort Handler\nhttp: Server closed\n200 418 429\nAccept-Encoding\nUser-Agent\nFoo-Bar_baz\nfoo bar\nOK\nI'm a teapot\n[]\n1 1 true\n0 0 false\ntext/plain; charset=utf-8\ntext/html; charset=utf-8\ntext/xml; charset=utf-8\nimage/png\napplication/wasm\napplication/octet-stream\n"
    );
}

#[test]
fn compiles_and_runs_net_http_parse_time() {
    let source = r#"
package main
import "fmt"
import "net/http"
import "time"

func main() {
    parsed, err := http.ParseTime("Sun, 06 Nov 1994 08:49:37 GMT")
    fmt.Println(err == nil, parsed.Format(time.DateTime))

    parsed, err = http.ParseTime("Sunday, 06-Nov-94 08:49:37 GMT")
    fmt.Println(err == nil, parsed.Format(time.DateTime))

    parsed, err = http.ParseTime("Sun Nov  6 08:49:37 1994")
    fmt.Println(err == nil, parsed.Format(time.DateTime))

    parsed, err = http.ParseTime("bad")
    fmt.Println(err != nil, parsed.IsZero())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true 1994-11-06 08:49:37\ntrue 1994-11-06 08:49:37\ntrue 1994-11-06 08:49:37\ntrue true\n"
    );
}

#[test]
fn compiles_and_runs_imported_net_http_request_and_header_types() {
    let source = r#"
package main
import "fmt"
import "net/http"
import "net/url"

func main() {
    var req http.Request
    fmt.Println(req.Method == "", req.URL == nil, len(req.Header), req.Body == nil)

    header := make(http.Header)
    header["Accept"] = []string{"text/plain"}
    header["X-Test"] = []string{"one", "two"}
    target := &url.URL{
        Scheme: "https",
        Host: "example.com",
        Path: "/submit",
        RawQuery: "q=1",
        Fragment: "frag",
    }

    req = http.Request{
        Method: http.MethodPost,
        URL: target,
        Header: header,
    }

    fmt.Println(req.Method)
    fmt.Println(len(req.Header), req.Header["Accept"][0], req.Header["X-Test"][1])
    fmt.Println(req.URL.Scheme, req.URL.Host, req.URL.Path, req.URL.RawQuery, req.URL.Fragment)
    fmt.Println(req.URL.String())

    converted := http.Header(map[string][]string{
        "Content-Type": []string{"text/plain"},
    })
    fmt.Println(len(converted), converted["Content-Type"][0])

    ptr := new(http.Request)
    ptr.Method = http.MethodGet
    ptr.URL = &url.URL{
        Path: "/api",
    }
    ptr.Header = http.Header(map[string][]string{
        "Accept": []string{"application/json"},
    })
    fmt.Println(ptr.Method, ptr.URL.Path, len(ptr.Header), ptr.Header["Accept"][0])
    fmt.Println(ptr.URL.String())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true true 0 true\nPOST\n2 text/plain two\nhttps example.com /submit q=1 frag\nhttps://example.com/submit?q=1#frag\n1 text/plain\nGET /api 1 application/json\n/api\n"
    );
}

#[test]
fn compiles_and_runs_imported_net_http_response_type() {
    let source = r#"
package main
import "fmt"
import "io"
import "net/http"

func main() {
    var closer io.Closer
    var readCloser io.ReadCloser
    var resp http.Response
    fmt.Println(closer == nil, readCloser == nil)
    fmt.Println(resp.Status == "", resp.StatusCode == 0, len(resp.Header), resp.Body == nil)

    header := make(http.Header)
    header["Content-Type"] = []string{"text/plain"}

    resp = http.Response{
        Status: "201 Created",
        StatusCode: http.StatusCreated,
        Header: header,
        Body: readCloser,
    }

    var body io.ReadCloser = resp.Body
    fmt.Println(resp.Status, resp.StatusCode, resp.Header["Content-Type"][0], body == nil)

    ptr := new(http.Response)
    ptr.Status = "404 Not Found"
    ptr.StatusCode = http.StatusNotFound
    ptr.Header = http.Header(map[string][]string{
        "X-Test": []string{"one", "two"},
    })
    ptr.Body = body
    fmt.Println(ptr.Status, ptr.StatusCode, len(ptr.Header), ptr.Header["X-Test"][1], ptr.Body == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true true\ntrue true 0 true\n201 Created 201 text/plain true\n404 Not Found 404 1 two true\n"
    );
}

#[test]
fn compiles_and_runs_net_http_header_methods() {
    let source = r#"
package main
import "fmt"
import "net/http"

func main() {
    var nilHeader http.Header
    fmt.Println(nilHeader.Get("Accept") == "", len(nilHeader.Values("Accept")))

    header := make(http.Header)
    fmt.Println(header.Get("Accept") == "", len(header.Values("Accept")))

    header.Set("accept", "text/plain")
    header.Add("ACCEPT", "application/json")
    fmt.Println(header.Get("Accept"))
    values := header.Values("accept")
    fmt.Println(len(values), values[0], values[1])
    cloned := header.Clone()
    cloned.Add("Accept", "text/html")
    clonedValues := cloned.Values("Accept")
    fmt.Println(len(header.Values("Accept")), len(clonedValues), clonedValues[2])
    header.Del("Accept")
    fmt.Println(header.Get("Accept") == "", len(header.Values("Accept")))

    req := http.Request{Header: make(http.Header)}
    req.Header.Set("content-type", "text/plain")
    req.Header.Add("Content-Type", "application/json")
    fmt.Println(req.Header.Get("CONTENT-TYPE"))
    reqValues := req.Header.Values("Content-Type")
    fmt.Println(len(reqValues), reqValues[1])
    req.Header.Del("Content-Type")
    fmt.Println(req.Header.Get("Content-Type") == "", len(req.Header.Values("Content-Type")))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true 0\ntrue 0\ntext/plain\n2 text/plain application/json\n2 3 text/html\ntrue 0\ntext/plain\n2 application/json\ntrue 0\n"
    );
}

#[test]
fn compiles_and_runs_net_http_new_request() {
    let source = r#"
package main
import "fmt"
import "net/http"

func main() {
    req, err := http.NewRequest("", "https://example.com/submit?q=1#frag", nil)
    fmt.Println(err == nil, req != nil)
    fmt.Println(req.Method, req.URL.String(), len(req.Header), req.Body == nil)

    req, err = http.NewRequest("POST", "/api", nil)
    fmt.Println(err == nil, req.Method, req.URL.Path, len(req.Header), req.Body == nil)

    req, err = http.NewRequest("bad method", "https://example.com", nil)
    fmt.Println(err != nil, req == nil)

    req, err = http.NewRequest("GET", "bad\nurl", nil)
    fmt.Println(err != nil, req == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true true\nGET https://example.com/submit?q=1#frag 0 true\ntrue POST /api 0 true\ntrue true\ntrue true\n"
    );
}

#[test]
fn compiles_and_runs_net_http_new_request_with_context() {
    let source = r#"
package main
import "context"
import "fmt"
import "net/http"

func main() {
    req, err := http.NewRequestWithContext(context.Background(), "", "/ctx", nil)
    fmt.Println(err == nil, req != nil)
    fmt.Println(req.Method, req.URL.Path, len(req.Header), req.Body == nil)

    req, err = http.NewRequestWithContext(nil, "GET", "/ctx", nil)
    fmt.Println(err != nil, req == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true\nGET /ctx 0 true\ntrue true\n");
}

#[test]
fn compiles_and_runs_net_http_request_context() {
    let source = r#"
package main
import "context"
import "fmt"
import "net/http"

func main() {
    req, _ := http.NewRequest("GET", "/plain", nil)
    plain := req.Context()
    deadline, ok := plain.Deadline()
    fmt.Println(ok, deadline.UnixNano(), plain.Err() == nil, plain.Value("k") == nil)

    ctx := context.WithValue(context.Background(), "k", "v")
    req, _ = http.NewRequestWithContext(ctx, "POST", "/ctx", nil)
    fmt.Println(req.Context().Value("k"))

    zero := new(http.Request)
    fmt.Println(zero.Context().Err() == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false 0 true true\nv\ntrue\n");
}

#[test]
fn compiles_and_runs_net_http_request_with_context() {
    let source = r#"
package main
import "context"
import "fmt"
import "net/http"

func main() {
    first := context.WithValue(context.Background(), "k", "first")
    req, _ := http.NewRequestWithContext(first, "GET", "/ctx", nil)

    second := context.WithValue(context.Background(), "k", "second")
    cloned := req.WithContext(second)

    fmt.Println(req.Context().Value("k"), cloned.Context().Value("k"))
    fmt.Println(req.URL.String(), cloned.URL.String(), req.Method, cloned.Method)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "first second\n/ctx /ctx GET GET\n");
}

#[test]
fn net_http_request_with_context_panics_on_nil_context() {
    let source = r#"
package main
import "net/http"

func main() {
    req, _ := http.NewRequest("GET", "/ctx", nil)
    req.WithContext(nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should panic");
    assert!(error.to_string().contains("nil context"));
}

#[test]
fn compiles_and_runs_net_http_request_clone() {
    let source = r#"
package main
import "context"
import "fmt"
import "net/http"

func main() {
    first := context.WithValue(context.Background(), "k", "first")
    req, _ := http.NewRequestWithContext(first, "GET", "/ctx", nil)
    req.Header.Set("X-Test", "one")

    second := context.WithValue(context.Background(), "k", "second")
    cloned := req.Clone(second)
    cloned.Header.Set("X-Test", "two")

    fmt.Println(req.Context().Value("k"), cloned.Context().Value("k"))
    fmt.Println(req.Header.Get("X-Test"), cloned.Header.Get("X-Test"))
    fmt.Println(req.URL == cloned.URL, req.URL.String(), cloned.URL.String(), req.Method, cloned.Method)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "first second\none two\nfalse /ctx /ctx GET GET\n"
    );
}

#[test]
fn net_http_request_clone_panics_on_nil_context() {
    let source = r#"
package main
import "net/http"

func main() {
    req, _ := http.NewRequest("GET", "/ctx", nil)
    req.Clone(nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should panic");
    assert!(error.to_string().contains("nil context"));
}

#[test]
fn net_http_get_uses_fetch_capability_and_builds_responses() {
    let source = r#"
package main
import "fmt"
import "net/http"

func main() {
    resp, err := http.Get("https://example.com/data?q=1")
    fmt.Println(err == nil, resp != nil)
    fmt.Println(resp.Status, resp.StatusCode, resp.Header.Get("Content-Type"), resp.Header.Values("X-Test")[1], resp.Body != nil)

    buf := make([]byte, 5)
    n, err := resp.Body.Read(buf)
    fmt.Println(n, string(buf[:n]), err == nil)

    err = resp.Body.Close()
    fmt.Println(err == nil)

    n, err = resp.Body.Read(buf)
    fmt.Println(n, err)
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
                    url: "https://example.com/data?q=1".into(),
                    headers: Vec::new(),
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
        headers: vec![
            FetchHeader {
                name: "content-type".into(),
                values: vec!["text/plain".into()],
            },
            FetchHeader {
                name: "x-test".into(),
                values: vec!["one".into(), "two".into()],
            },
        ],
        body: b"hello".to_vec(),
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
        "true true\n202 Accepted 202 text/plain two true\n5 hello true\ntrue\n0 http: read on closed response body\n"
    );
}

#[test]
fn net_http_post_uses_fetch_capability_and_buffers_file_bodies() {
    let source = r#"
package main
import "fmt"
import "net/http"
import "os"

func main() {
    file, fileErr := os.DirFS("assets").Open("payload.txt")
    fmt.Println(fileErr == nil)

    resp, err := http.Post("https://example.com/upload?q=1", "text/plain", file)
    fmt.Println(err == nil, resp != nil)
    fmt.Println(resp.Status, resp.StatusCode, resp.Header.Get("X-Reply"), resp.Body != nil)

    buf := make([]byte, 3)
    n, err := resp.Body.Read(buf)
    fmt.Println(n, string(buf[:n]), err == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files
        .insert("assets/payload.txt".into(), "hello".into());
    vm.enable_capability_requests();

    complete_streamed_fetch_with_buffered_response(
        &mut vm,
        &program,
        FetchStartRequest {
            session_id: 0,
            method: "POST".into(),
            url: "https://example.com/upload?q=1".into(),
            headers: vec![FetchHeader {
                name: "Content-Type".into(),
                values: vec!["text/plain".into()],
            }],
            context_deadline_unix_millis: None,
        },
        &[b"hello"],
        FetchResponse {
            status_code: 201,
            status: "201 Created".into(),
            url: String::new(),
            headers: vec![FetchHeader {
                name: "x-reply".into(),
                values: vec!["ok".into()],
            }],
            body: b"yes".to_vec(),
        },
    );

    assert_eq!(
        vm.stdout(),
        "true\ntrue true\n201 Created 201 ok true\n3 yes true\n"
    );
}

#[test]
fn net_http_post_buffers_custom_reader_bodies() {
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
    reader := &customReader{data: "chunked"}
    resp, err := http.Post("https://example.com/upload", "text/plain", reader)
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
fn net_http_client_do_uses_request_values() {
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
    if len(remaining) > len(p) {
        remaining = remaining[:len(p)]
    }

    n := copy(p, remaining)
    r.offset += n
    if r.offset >= len(r.data) {
        return n, errors.New("EOF")
    }
    return n, nil
}

func main() {
    client := new(http.Client)
    req, reqErr := http.NewRequest("PATCH", "https://example.com/api?q=1#frag", &customReader{data: "payload"})
    fmt.Println(reqErr == nil, req != nil)

    req.Header.Add("X-Test", "one")
    req.Header.Add("x-test", "two")
    req.Header.Set("Content-Type", "text/plain")

    resp, err := client.Do(req)
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

    complete_streamed_fetch_with_buffered_response(
        &mut vm,
        &program,
        FetchStartRequest {
            session_id: 0,
            method: "PATCH".into(),
            url: "https://example.com/api?q=1#frag".into(),
            headers: vec![
                FetchHeader {
                    name: "Content-Type".into(),
                    values: vec!["text/plain".into()],
                },
                FetchHeader {
                    name: "X-Test".into(),
                    values: vec!["one".into(), "two".into()],
                },
            ],
            context_deadline_unix_millis: None,
        },
        &[b"payload"],
        FetchResponse {
            status_code: 202,
            status: "202 Accepted".into(),
            url: String::new(),
            headers: vec![FetchHeader {
                name: "x-reply".into(),
                values: vec!["ok".into()],
            }],
            body: b"done".to_vec(),
        },
    );

    assert_eq!(
        vm.stdout(),
        "true true\ntrue true\n202 Accepted 202 ok true\n4 done true\n"
    );
}
