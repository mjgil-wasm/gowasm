use gowasm_vm::{CapabilityRequest, FetchRequest, FetchResponse, RunOutcome, Vm};

use crate::compile_source;

#[test]
fn net_http_request_body_reuse_survives_with_context_and_clone() {
    let source = r#"
package main
import "context"
import "errors"
import "fmt"
import "net/http"

type sharedReader struct {
    data string
    state []int
}

func (r sharedReader) Read(p []byte) (int, error) {
    if r.state[0] >= len(r.data) {
        return 0, errors.New("EOF")
    }

    remaining := []byte(r.data[r.state[0]:])
    if len(remaining) > len(p) {
        remaining = remaining[:len(p)]
    }

    n := copy(p, remaining)
    state := r.state
    state[0] = state[0] + n
    if r.state[0] >= len(r.data) {
        return n, errors.New("EOF")
    }
    return n, nil
}

func main() {
    req, _ := http.NewRequest("POST", "/reuse", sharedReader{
        data: "chunked",
        state: []int{0},
    })
    withCtx := req.WithContext(context.Background())
    cloned := req.Clone(context.Background())

    buf := make([]byte, 3)

    n, err := req.Body.Read(buf)
    fmt.Println(n, string(buf[:n]), err == nil)

    n, err = withCtx.Body.Read(buf)
    fmt.Println(n, string(buf[:n]), err == nil)

    n, err = cloned.Body.Read(buf)
    fmt.Println(n, string(buf[:n]), err)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "3 chu true\n3 nke true\n1 d EOF\n");
}

#[test]
fn net_http_response_copies_share_body_progress_and_close_state() {
    let source = r#"
package main
import "fmt"
import "net/http"

func main() {
    resp, _ := http.Get("https://example.com/data")
    copied := *resp

    buf := make([]byte, 3)

    n, err := resp.Body.Read(buf)
    fmt.Println(n, string(buf[:n]), err == nil)

    n, err = copied.Body.Read(buf)
    fmt.Println(n, string(buf[:n]), err)

    err = copied.Body.Close()
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
                    url: "https://example.com/data".into(),
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
        url: "https://example.com/data".into(),
        headers: Vec::new(),
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
        "3 hel true\n2 lo <nil>\ntrue\n0 http: read on closed response body\n"
    );
}
