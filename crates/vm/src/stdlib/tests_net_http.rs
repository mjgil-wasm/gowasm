use super::{
    resolve_stdlib_function, resolve_stdlib_method, resolve_stdlib_runtime_method,
    stdlib_function_mutates_first_arg, stdlib_function_param_types, stdlib_function_result_count,
    stdlib_function_result_types, stdlib_function_returns_value,
};
use crate::{
    Function, Instruction, Program, RunOutcome, Vm, TYPE_HTTP_CLIENT, TYPE_HTTP_CLIENT_PTR,
    TYPE_HTTP_HEADER, TYPE_HTTP_REQUEST_BODY, TYPE_HTTP_REQUEST_PTR, TYPE_HTTP_RESPONSE_BODY,
};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should exist")
}

fn http_new_request() -> super::StdlibFunctionId {
    resolve_stdlib_function("net/http", "NewRequest").expect("http.NewRequest should exist")
}

fn http_post() -> super::StdlibFunctionId {
    resolve_stdlib_function("net/http", "Post").expect("http.Post should exist")
}

fn http_post_form() -> super::StdlibFunctionId {
    resolve_stdlib_function("net/http", "PostForm").expect("http.PostForm should exist")
}

fn http_client_do() -> super::StdlibFunctionId {
    resolve_stdlib_method("*http.Client", "Do").expect("(*http.Client).Do should exist")
}

fn http_client_get() -> super::StdlibFunctionId {
    resolve_stdlib_method("*http.Client", "Get").expect("(*http.Client).Get should exist")
}

fn http_client_head() -> super::StdlibFunctionId {
    resolve_stdlib_method("*http.Client", "Head").expect("(*http.Client).Head should exist")
}

fn http_client_post() -> super::StdlibFunctionId {
    resolve_stdlib_method("*http.Client", "Post").expect("(*http.Client).Post should exist")
}

fn http_client_post_form() -> super::StdlibFunctionId {
    resolve_stdlib_method("*http.Client", "PostForm").expect("(*http.Client).PostForm should exist")
}

fn http_response_location() -> super::StdlibFunctionId {
    resolve_stdlib_method("*http.Response", "Location")
        .expect("(*http.Response).Location should exist")
}

#[test]
fn resolves_net_http_new_request_from_the_registry() {
    let get = resolve_stdlib_function("net/http", "Get").expect("http.Get should exist");
    assert!(!stdlib_function_returns_value(get));
    assert_eq!(stdlib_function_result_count(get), 2);
    assert_eq!(stdlib_function_param_types(get), Some(&["string"][..]));
    assert_eq!(
        stdlib_function_result_types(get),
        Some(&["*http.Response", "error"][..])
    );

    let head = resolve_stdlib_function("net/http", "Head").expect("http.Head should exist");
    assert!(!stdlib_function_returns_value(head));
    assert_eq!(stdlib_function_result_count(head), 2);
    assert_eq!(stdlib_function_param_types(head), Some(&["string"][..]));
    assert_eq!(
        stdlib_function_result_types(head),
        Some(&["*http.Response", "error"][..])
    );

    let post = resolve_stdlib_function("net/http", "Post").expect("http.Post should exist");
    assert!(!stdlib_function_returns_value(post));
    assert_eq!(stdlib_function_result_count(post), 2);
    assert_eq!(
        stdlib_function_param_types(post),
        Some(&["string", "string", "io.Reader"][..])
    );
    assert_eq!(
        stdlib_function_result_types(post),
        Some(&["*http.Response", "error"][..])
    );

    let post_form = http_post_form();
    assert!(!stdlib_function_returns_value(post_form));
    assert_eq!(stdlib_function_result_count(post_form), 2);
    assert_eq!(
        stdlib_function_param_types(post_form),
        Some(&["string", "url.Values"][..])
    );
    assert_eq!(
        stdlib_function_result_types(post_form),
        Some(&["*http.Response", "error"][..])
    );

    let new_request =
        resolve_stdlib_function("net/http", "NewRequest").expect("http.NewRequest should exist");
    assert!(!stdlib_function_returns_value(new_request));
    assert_eq!(stdlib_function_result_count(new_request), 2);
    assert_eq!(
        stdlib_function_param_types(new_request),
        Some(&["string", "string", "io.Reader"][..])
    );
    assert_eq!(
        stdlib_function_result_types(new_request),
        Some(&["*http.Request", "error"][..])
    );

    let with_context = resolve_stdlib_function("net/http", "NewRequestWithContext")
        .expect("http.NewRequestWithContext should exist");
    assert!(!stdlib_function_returns_value(with_context));
    assert_eq!(stdlib_function_result_count(with_context), 2);
    assert_eq!(
        stdlib_function_param_types(with_context),
        Some(&["context.Context", "string", "string", "io.Reader"][..])
    );
    assert_eq!(
        stdlib_function_result_types(with_context),
        Some(&["*http.Request", "error"][..])
    );
}

#[test]
fn resolves_net_http_header_methods_from_the_registry() {
    let get = resolve_stdlib_method("http.Header", "Get").expect("http.Header.Get should exist");
    assert!(stdlib_function_returns_value(get));
    assert_eq!(stdlib_function_result_count(get), 1);
    assert_eq!(
        stdlib_function_param_types(get),
        Some(&["http.Header", "string"][..])
    );
    assert_eq!(stdlib_function_result_types(get), Some(&["string"][..]));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_HTTP_HEADER, "Get"),
        Some(get)
    );

    let values =
        resolve_stdlib_method("http.Header", "Values").expect("http.Header.Values should exist");
    assert!(stdlib_function_returns_value(values));
    assert_eq!(stdlib_function_result_count(values), 1);
    assert_eq!(
        stdlib_function_param_types(values),
        Some(&["http.Header", "string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(values),
        Some(&["[]string"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_HTTP_HEADER, "Values"),
        Some(values)
    );

    let clone =
        resolve_stdlib_method("http.Header", "Clone").expect("http.Header.Clone should exist");
    assert!(stdlib_function_returns_value(clone));
    assert_eq!(stdlib_function_result_count(clone), 1);
    assert_eq!(
        stdlib_function_param_types(clone),
        Some(&["http.Header"][..])
    );
    assert_eq!(
        stdlib_function_result_types(clone),
        Some(&["http.Header"][..])
    );
    assert!(!stdlib_function_mutates_first_arg(clone));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_HTTP_HEADER, "Clone"),
        Some(clone)
    );

    let set = resolve_stdlib_method("http.Header", "Set").expect("http.Header.Set should exist");
    assert!(!stdlib_function_returns_value(set));
    assert_eq!(stdlib_function_result_count(set), 0);
    assert_eq!(
        stdlib_function_param_types(set),
        Some(&["http.Header", "string", "string"][..])
    );
    assert_eq!(stdlib_function_result_types(set), None);
    assert!(stdlib_function_mutates_first_arg(set));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_HTTP_HEADER, "Set"),
        Some(set)
    );

    let add = resolve_stdlib_method("http.Header", "Add").expect("http.Header.Add should exist");
    assert!(!stdlib_function_returns_value(add));
    assert_eq!(stdlib_function_result_count(add), 0);
    assert_eq!(
        stdlib_function_param_types(add),
        Some(&["http.Header", "string", "string"][..])
    );
    assert_eq!(stdlib_function_result_types(add), None);
    assert!(stdlib_function_mutates_first_arg(add));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_HTTP_HEADER, "Add"),
        Some(add)
    );

    let del = resolve_stdlib_method("http.Header", "Del").expect("http.Header.Del should exist");
    assert!(!stdlib_function_returns_value(del));
    assert_eq!(stdlib_function_result_count(del), 0);
    assert_eq!(
        stdlib_function_param_types(del),
        Some(&["http.Header", "string"][..])
    );
    assert_eq!(stdlib_function_result_types(del), None);
    assert!(stdlib_function_mutates_first_arg(del));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_HTTP_HEADER, "Del"),
        Some(del)
    );
}

#[test]
fn resolves_net_http_request_methods_from_the_registry() {
    let context =
        resolve_stdlib_method("*http.Request", "Context").expect("(*http.Request).Context exists");
    assert!(stdlib_function_returns_value(context));
    assert_eq!(stdlib_function_result_count(context), 1);
    assert_eq!(
        stdlib_function_param_types(context),
        Some(&["*http.Request"][..])
    );
    assert_eq!(
        stdlib_function_result_types(context),
        Some(&["context.Context"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_HTTP_REQUEST_PTR, "Context"),
        Some(context)
    );

    let with_context = resolve_stdlib_method("*http.Request", "WithContext")
        .expect("(*http.Request).WithContext exists");
    assert!(stdlib_function_returns_value(with_context));
    assert_eq!(stdlib_function_result_count(with_context), 1);
    assert_eq!(
        stdlib_function_param_types(with_context),
        Some(&["*http.Request", "context.Context"][..])
    );
    assert_eq!(
        stdlib_function_result_types(with_context),
        Some(&["*http.Request"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_HTTP_REQUEST_PTR, "WithContext"),
        Some(with_context)
    );

    let clone =
        resolve_stdlib_method("*http.Request", "Clone").expect("(*http.Request).Clone exists");
    assert!(stdlib_function_returns_value(clone));
    assert_eq!(stdlib_function_result_count(clone), 1);
    assert_eq!(
        stdlib_function_param_types(clone),
        Some(&["*http.Request", "context.Context"][..])
    );
    assert_eq!(
        stdlib_function_result_types(clone),
        Some(&["*http.Request"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_HTTP_REQUEST_PTR, "Clone"),
        Some(clone)
    );
}

#[test]
fn resolves_net_http_response_methods_from_the_registry() {
    let location = http_response_location();
    assert!(!stdlib_function_returns_value(location));
    assert_eq!(stdlib_function_result_count(location), 2);
    assert_eq!(
        stdlib_function_param_types(location),
        Some(&["*http.Response"][..])
    );
    assert_eq!(
        stdlib_function_result_types(location),
        Some(&["*url.URL", "error"][..])
    );
}

#[test]
fn resolves_net_http_client_transport_methods_from_the_registry() {
    let do_method = http_client_do();
    assert!(!stdlib_function_returns_value(do_method));
    assert_eq!(stdlib_function_result_count(do_method), 2);
    assert_eq!(
        stdlib_function_param_types(do_method),
        Some(&["*http.Client", "*http.Request"][..])
    );
    assert_eq!(
        stdlib_function_result_types(do_method),
        Some(&["*http.Response", "error"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_HTTP_CLIENT_PTR, "Do"),
        Some(do_method)
    );

    let get_method = http_client_get();
    assert!(!stdlib_function_returns_value(get_method));
    assert_eq!(stdlib_function_result_count(get_method), 2);
    assert_eq!(
        stdlib_function_param_types(get_method),
        Some(&["*http.Client", "string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(get_method),
        Some(&["*http.Response", "error"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_HTTP_CLIENT_PTR, "Get"),
        Some(get_method)
    );

    let head_method = http_client_head();
    assert!(!stdlib_function_returns_value(head_method));
    assert_eq!(stdlib_function_result_count(head_method), 2);
    assert_eq!(
        stdlib_function_param_types(head_method),
        Some(&["*http.Client", "string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(head_method),
        Some(&["*http.Response", "error"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_HTTP_CLIENT_PTR, "Head"),
        Some(head_method)
    );

    let post_method = http_client_post();
    assert!(!stdlib_function_returns_value(post_method));
    assert_eq!(stdlib_function_result_count(post_method), 2);
    assert_eq!(
        stdlib_function_param_types(post_method),
        Some(&["*http.Client", "string", "string", "io.Reader"][..])
    );
    assert_eq!(
        stdlib_function_result_types(post_method),
        Some(&["*http.Response", "error"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_HTTP_CLIENT_PTR, "Post"),
        Some(post_method)
    );

    let post_form_method = http_client_post_form();
    assert!(!stdlib_function_returns_value(post_form_method));
    assert_eq!(stdlib_function_result_count(post_form_method), 2);
    assert_eq!(
        stdlib_function_param_types(post_form_method),
        Some(&["*http.Client", "string", "url.Values"][..])
    );
    assert_eq!(
        stdlib_function_result_types(post_form_method),
        Some(&["*http.Response", "error"][..])
    );
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_HTTP_CLIENT_PTR, "PostForm"),
        Some(post_form_method)
    );
}

#[test]
fn resolves_net_http_request_body_methods_from_the_registry() {
    let read = resolve_stdlib_method("http.__requestBody", "Read")
        .expect("http.__requestBody.Read should exist");
    assert!(!stdlib_function_returns_value(read));
    assert_eq!(stdlib_function_result_count(read), 2);
    assert_eq!(
        stdlib_function_param_types(read),
        Some(&["http.__requestBody", "[]byte"][..])
    );
    assert_eq!(
        stdlib_function_result_types(read),
        Some(&["int", "error"][..])
    );
    assert!(stdlib_function_mutates_first_arg(read));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_HTTP_REQUEST_BODY, "Read"),
        Some(read)
    );
}

#[test]
fn resolves_net_http_response_body_methods_from_the_registry() {
    let read = resolve_stdlib_method("http.__responseBody", "Read")
        .expect("http.__responseBody.Read should exist");
    assert!(!stdlib_function_returns_value(read));
    assert_eq!(stdlib_function_result_count(read), 2);
    assert_eq!(
        stdlib_function_param_types(read),
        Some(&["http.__responseBody", "[]byte"][..])
    );
    assert_eq!(
        stdlib_function_result_types(read),
        Some(&["int", "error"][..])
    );
    assert!(stdlib_function_mutates_first_arg(read));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_HTTP_RESPONSE_BODY, "Read"),
        Some(read)
    );

    let close = resolve_stdlib_method("http.__responseBody", "Close")
        .expect("http.__responseBody.Close should exist");
    assert!(stdlib_function_returns_value(close));
    assert_eq!(stdlib_function_result_count(close), 1);
    assert_eq!(
        stdlib_function_param_types(close),
        Some(&["http.__responseBody"][..])
    );
    assert_eq!(stdlib_function_result_types(close), Some(&["error"][..]));
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_HTTP_RESPONSE_BODY, "Close"),
        Some(close)
    );
}

#[test]
fn net_http_transport_rejects_non_reader_bodies_before_fetch_requests() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 14,
            code: vec![
                Instruction::LoadString {
                    dst: 0,
                    value: "PUT".into(),
                },
                Instruction::LoadString {
                    dst: 1,
                    value: "https://example.com/upload".into(),
                },
                Instruction::LoadString {
                    dst: 2,
                    value: "text/plain".into(),
                },
                Instruction::LoadInt { dst: 3, value: 7 },
                Instruction::CallStdlibMulti {
                    function: http_new_request(),
                    args: vec![0, 1, 3],
                    dsts: vec![4, 5],
                },
                Instruction::MakeStruct {
                    dst: 6,
                    typ: TYPE_HTTP_CLIENT,
                    fields: vec![],
                },
                Instruction::BoxHeap {
                    dst: 7,
                    src: 6,
                    typ: TYPE_HTTP_CLIENT_PTR,
                },
                Instruction::CallStdlibMulti {
                    function: http_client_do(),
                    args: vec![7, 4],
                    dsts: vec![8, 9],
                },
                Instruction::CallStdlibMulti {
                    function: http_post(),
                    args: vec![1, 2, 3],
                    dsts: vec![10, 11],
                },
                Instruction::CallStdlibMulti {
                    function: http_client_post(),
                    args: vec![7, 1, 2, 3],
                    dsts: vec![12, 13],
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![8, 9],
                    dst: None,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![10, 11],
                    dst: None,
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![12, 13],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.enable_capability_requests();

    match vm
        .start_program(&program)
        .expect("program should complete without a fetch capability request")
    {
        RunOutcome::Completed => {}
        other => panic!("unexpected run outcome: {other:?}"),
    }

    assert_eq!(
        vm.stdout(),
        "<nil> net/http: (*http.Client).Do body must implement io.Reader\n\
<nil> net/http: http.Post body must implement io.Reader\n\
<nil> net/http: (*http.Client).Post body must implement io.Reader\n"
    );
}
