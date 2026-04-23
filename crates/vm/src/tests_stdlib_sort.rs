use super::{resolve_stdlib_function, Function, Instruction, Program, Vm};

fn fmt_println() -> super::StdlibFunctionId {
    resolve_stdlib_function("fmt", "Println").expect("fmt.Println should be registered")
}

fn sort_ints_are_sorted() -> super::StdlibFunctionId {
    resolve_stdlib_function("sort", "IntsAreSorted").expect("sort.IntsAreSorted should exist")
}

fn sort_strings_are_sorted() -> super::StdlibFunctionId {
    resolve_stdlib_function("sort", "StringsAreSorted").expect("sort.StringsAreSorted should exist")
}

fn sort_search_ints() -> super::StdlibFunctionId {
    resolve_stdlib_function("sort", "SearchInts").expect("sort.SearchInts should exist")
}

fn sort_search_strings() -> super::StdlibFunctionId {
    resolve_stdlib_function("sort", "SearchStrings").expect("sort.SearchStrings should exist")
}

#[test]
fn resolves_mutation_capable_sort_helpers() {
    let ints = resolve_stdlib_function("sort", "Ints").expect("sort.Ints should exist");
    let strings = resolve_stdlib_function("sort", "Strings").expect("sort.Strings should exist");
    let float64s = resolve_stdlib_function("sort", "Float64s").expect("sort.Float64s should exist");
    let float64s_are_sorted = resolve_stdlib_function("sort", "Float64sAreSorted")
        .expect("sort.Float64sAreSorted should exist");
    let search_float64s = resolve_stdlib_function("sort", "SearchFloat64s")
        .expect("sort.SearchFloat64s should exist");
    let slice = resolve_stdlib_function("sort", "Slice").expect("sort.Slice should exist");
    let slice_is_sorted =
        resolve_stdlib_function("sort", "SliceIsSorted").expect("sort.SliceIsSorted should exist");
    let slice_stable =
        resolve_stdlib_function("sort", "SliceStable").expect("sort.SliceStable should exist");
    let search = resolve_stdlib_function("sort", "Search").expect("sort.Search should exist");

    assert_ne!(ints.0, strings.0);
    assert_ne!(float64s.0, float64s_are_sorted.0);
    assert_ne!(search_float64s.0, search.0);
    assert_ne!(slice.0, slice_is_sorted.0);
    assert_ne!(slice_is_sorted.0, slice_stable.0);
}

#[test]
fn executes_sort_queries() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 28,
            code: vec![
                Instruction::LoadInt { dst: 0, value: 1 },
                Instruction::LoadInt { dst: 1, value: 2 },
                Instruction::LoadInt { dst: 2, value: 4 },
                Instruction::LoadInt { dst: 3, value: 4 },
                Instruction::LoadInt { dst: 4, value: 9 },
                Instruction::MakeSlice {
                    concrete_type: None,
                    dst: 5,
                    items: vec![0, 1, 2, 3, 4],
                },
                Instruction::LoadInt { dst: 6, value: 3 },
                Instruction::LoadInt { dst: 7, value: 1 },
                Instruction::MakeSlice {
                    concrete_type: None,
                    dst: 8,
                    items: vec![6, 7],
                },
                Instruction::LoadString {
                    dst: 9,
                    value: "ant".into(),
                },
                Instruction::LoadString {
                    dst: 10,
                    value: "bee".into(),
                },
                Instruction::LoadString {
                    dst: 11,
                    value: "cat".into(),
                },
                Instruction::MakeSlice {
                    concrete_type: None,
                    dst: 12,
                    items: vec![9, 10, 11],
                },
                Instruction::LoadString {
                    dst: 13,
                    value: "bee".into(),
                },
                Instruction::LoadString {
                    dst: 14,
                    value: "ant".into(),
                },
                Instruction::MakeSlice {
                    concrete_type: None,
                    dst: 15,
                    items: vec![13, 14],
                },
                Instruction::LoadInt { dst: 16, value: 4 },
                Instruction::LoadInt { dst: 17, value: 5 },
                Instruction::LoadString {
                    dst: 18,
                    value: "bee".into(),
                },
                Instruction::LoadString {
                    dst: 19,
                    value: "cow".into(),
                },
                Instruction::CallStdlib {
                    function: sort_ints_are_sorted(),
                    args: vec![5],
                    dst: Some(20),
                },
                Instruction::CallStdlib {
                    function: sort_ints_are_sorted(),
                    args: vec![8],
                    dst: Some(21),
                },
                Instruction::CallStdlib {
                    function: sort_search_ints(),
                    args: vec![5, 16],
                    dst: Some(22),
                },
                Instruction::CallStdlib {
                    function: sort_search_ints(),
                    args: vec![5, 17],
                    dst: Some(23),
                },
                Instruction::CallStdlib {
                    function: sort_strings_are_sorted(),
                    args: vec![12],
                    dst: Some(24),
                },
                Instruction::CallStdlib {
                    function: sort_strings_are_sorted(),
                    args: vec![15],
                    dst: Some(25),
                },
                Instruction::CallStdlib {
                    function: sort_search_strings(),
                    args: vec![12, 18],
                    dst: Some(26),
                },
                Instruction::CallStdlib {
                    function: sort_search_strings(),
                    args: vec![12, 19],
                    dst: Some(27),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![20, 21, 22, 23, 24, 25, 26, 27],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false 2 4 true false 1 3\n");
}

#[test]
fn executes_sort_nil_slice_queries() {
    let program = Program {
        entry_function: 0,
        methods: vec![],
        global_count: 0,
        functions: vec![Function {
            name: "main".into(),
            param_count: 0,
            register_count: 8,
            code: vec![
                Instruction::LoadNilSlice {
                    dst: 0,
                    concrete_type: None,
                },
                Instruction::LoadNilSlice {
                    dst: 1,
                    concrete_type: None,
                },
                Instruction::LoadInt { dst: 2, value: 7 },
                Instruction::LoadString {
                    dst: 3,
                    value: "go".into(),
                },
                Instruction::CallStdlib {
                    function: sort_ints_are_sorted(),
                    args: vec![0],
                    dst: Some(4),
                },
                Instruction::CallStdlib {
                    function: sort_strings_are_sorted(),
                    args: vec![1],
                    dst: Some(5),
                },
                Instruction::CallStdlib {
                    function: sort_search_ints(),
                    args: vec![0, 2],
                    dst: Some(6),
                },
                Instruction::CallStdlib {
                    function: sort_search_strings(),
                    args: vec![1, 3],
                    dst: Some(7),
                },
                Instruction::CallStdlib {
                    function: fmt_println(),
                    args: vec![4, 5, 6, 7],
                    dst: None,
                },
                Instruction::Return { src: None },
            ],
        }],
    };

    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true 0 0\n");
}
