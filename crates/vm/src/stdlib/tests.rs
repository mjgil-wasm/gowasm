use super::{
    resolve_stdlib_constant, resolve_stdlib_function, resolve_stdlib_value,
    stdlib_function_param_types, stdlib_function_result_count, stdlib_function_result_types,
    stdlib_function_returns_value, stdlib_function_variadic_param_type, StdlibValueInit,
};

#[test]
fn resolves_fmt_println_from_the_registry() {
    let function = resolve_stdlib_function("fmt", "Println").expect("fmt.Println should exist");
    assert!(!stdlib_function_returns_value(function));
    let printf = resolve_stdlib_function("fmt", "Printf").expect("fmt.Printf should exist");
    assert!(!stdlib_function_returns_value(printf));
    let sprintf = resolve_stdlib_function("fmt", "Sprintf").expect("fmt.Sprintf should exist");
    assert!(stdlib_function_returns_value(sprintf));
    let base = resolve_stdlib_function("path", "Base").expect("path.Base should exist");
    assert!(stdlib_function_returns_value(base));
    let clean = resolve_stdlib_function("path", "Clean").expect("path.Clean should exist");
    assert!(stdlib_function_returns_value(clean));
    let dir = resolve_stdlib_function("path", "Dir").expect("path.Dir should exist");
    assert!(stdlib_function_returns_value(dir));
    let ext = resolve_stdlib_function("path", "Ext").expect("path.Ext should exist");
    assert!(stdlib_function_returns_value(ext));
    let is_abs = resolve_stdlib_function("path", "IsAbs").expect("path.IsAbs should exist");
    assert!(stdlib_function_returns_value(is_abs));
    let split = resolve_stdlib_function("path", "Split").expect("path.Split should exist");
    assert!(!stdlib_function_returns_value(split));
    assert_eq!(stdlib_function_result_count(split), 2);
    assert_eq!(
        stdlib_function_result_types(split),
        Some(&["string", "string"][..])
    );
    let join = resolve_stdlib_function("path", "Join").expect("path.Join should exist");
    assert!(stdlib_function_returns_value(join));
    let path_match = resolve_stdlib_function("path", "Match").expect("path.Match should exist");
    assert!(!stdlib_function_returns_value(path_match));
    assert_eq!(stdlib_function_result_count(path_match), 2);
    assert_eq!(
        stdlib_function_result_types(path_match),
        Some(&["bool", "error"][..])
    );
    let filepath_base =
        resolve_stdlib_function("path/filepath", "Base").expect("filepath.Base should exist");
    assert!(stdlib_function_returns_value(filepath_base));
    let filepath_split =
        resolve_stdlib_function("path/filepath", "Split").expect("filepath.Split should exist");
    assert!(!stdlib_function_returns_value(filepath_split));
    assert_eq!(stdlib_function_result_count(filepath_split), 2);
    assert_eq!(
        stdlib_function_result_types(filepath_split),
        Some(&["string", "string"][..])
    );
    let filepath_join =
        resolve_stdlib_function("path/filepath", "Join").expect("filepath.Join should exist");
    assert!(stdlib_function_returns_value(filepath_join));
    assert_eq!(
        stdlib_function_variadic_param_type(filepath_join),
        Some("string")
    );
    let filepath_match =
        resolve_stdlib_function("path/filepath", "Match").expect("filepath.Match should exist");
    assert!(!stdlib_function_returns_value(filepath_match));
    assert_eq!(stdlib_function_result_count(filepath_match), 2);
    assert_eq!(
        stdlib_function_result_types(filepath_match),
        Some(&["bool", "error"][..])
    );
    let filepath_to_slash =
        resolve_stdlib_function("path/filepath", "ToSlash").expect("filepath.ToSlash should exist");
    assert!(stdlib_function_returns_value(filepath_to_slash));
    assert_eq!(
        stdlib_function_result_types(filepath_to_slash),
        Some(&["string"][..])
    );
    let filepath_from_slash = resolve_stdlib_function("path/filepath", "FromSlash")
        .expect("filepath.FromSlash should exist");
    assert!(stdlib_function_returns_value(filepath_from_slash));
    assert_eq!(
        stdlib_function_result_types(filepath_from_slash),
        Some(&["string"][..])
    );
    let filepath_split_list = resolve_stdlib_function("path/filepath", "SplitList")
        .expect("filepath.SplitList should exist");
    assert!(stdlib_function_returns_value(filepath_split_list));
    assert_eq!(
        stdlib_function_result_types(filepath_split_list),
        Some(&["[]string"][..])
    );
    let filepath_volume_name = resolve_stdlib_function("path/filepath", "VolumeName")
        .expect("filepath.VolumeName should exist");
    assert!(stdlib_function_returns_value(filepath_volume_name));
    assert_eq!(
        stdlib_function_result_types(filepath_volume_name),
        Some(&["string"][..])
    );
    let filepath_rel =
        resolve_stdlib_function("path/filepath", "Rel").expect("filepath.Rel should exist");
    assert!(!stdlib_function_returns_value(filepath_rel));
    assert_eq!(stdlib_function_result_count(filepath_rel), 2);
    assert_eq!(
        stdlib_function_result_types(filepath_rel),
        Some(&["string", "error"][..])
    );
    let filepath_is_local =
        resolve_stdlib_function("path/filepath", "IsLocal").expect("filepath.IsLocal should exist");
    assert!(stdlib_function_returns_value(filepath_is_local));
    assert_eq!(stdlib_function_result_count(filepath_is_local), 1);
    assert_eq!(
        stdlib_function_result_types(filepath_is_local),
        Some(&["bool"][..])
    );
    let filepath_localize = resolve_stdlib_function("path/filepath", "Localize")
        .expect("filepath.Localize should exist");
    assert!(!stdlib_function_returns_value(filepath_localize));
    assert_eq!(stdlib_function_result_count(filepath_localize), 2);
    assert_eq!(
        stdlib_function_result_types(filepath_localize),
        Some(&["string", "error"][..])
    );
    let filepath_glob =
        resolve_stdlib_function("path/filepath", "Glob").expect("filepath.Glob should exist");
    assert!(!stdlib_function_returns_value(filepath_glob));
    assert_eq!(stdlib_function_result_count(filepath_glob), 2);
    assert_eq!(
        stdlib_function_result_types(filepath_glob),
        Some(&["[]string", "error"][..])
    );
    let filepath_abs =
        resolve_stdlib_function("path/filepath", "Abs").expect("filepath.Abs should exist");
    assert!(!stdlib_function_returns_value(filepath_abs));
    assert_eq!(stdlib_function_result_count(filepath_abs), 2);
    assert_eq!(
        stdlib_function_result_types(filepath_abs),
        Some(&["string", "error"][..])
    );
    let filepath_walk_dir =
        resolve_stdlib_function("path/filepath", "WalkDir").expect("filepath.WalkDir should exist");
    assert!(stdlib_function_returns_value(filepath_walk_dir));
    assert_eq!(stdlib_function_result_count(filepath_walk_dir), 1);
    assert_eq!(
        stdlib_function_param_types(filepath_walk_dir),
        Some(
            &[
                "string",
                "__gowasm_func__(string, fs.DirEntry, error)->(error)",
            ][..]
        )
    );
    assert_eq!(
        stdlib_function_result_types(filepath_walk_dir),
        Some(&["error"][..])
    );
    let filepath_walk =
        resolve_stdlib_function("path/filepath", "Walk").expect("filepath.Walk should exist");
    assert!(stdlib_function_returns_value(filepath_walk));
    assert_eq!(stdlib_function_result_count(filepath_walk), 1);
    assert_eq!(
        stdlib_function_param_types(filepath_walk),
        Some(
            &[
                "string",
                "__gowasm_func__(string, fs.FileInfo, error)->(error)",
            ][..]
        )
    );
    assert_eq!(
        stdlib_function_result_types(filepath_walk),
        Some(&["error"][..])
    );
    let filepath_skip_dir =
        resolve_stdlib_value("path/filepath", "SkipDir").expect("filepath.SkipDir should exist");
    assert_eq!(filepath_skip_dir.typ, "error");
    let filepath_skip_all =
        resolve_stdlib_value("path/filepath", "SkipAll").expect("filepath.SkipAll should exist");
    assert_eq!(filepath_skip_all.typ, "error");
    let http_time_format =
        resolve_stdlib_constant("net/http", "TimeFormat").expect("http.TimeFormat should exist");
    assert_eq!(http_time_format.typ, "string");
    assert_eq!(
        http_time_format.value,
        super::StdlibConstantValue::String("Mon, 02 Jan 2006 15:04:05 GMT")
    );
    let trailer_prefix = resolve_stdlib_constant("net/http", "TrailerPrefix")
        .expect("http.TrailerPrefix should exist");
    assert_eq!(trailer_prefix.typ, "string");
    assert_eq!(
        trailer_prefix.value,
        super::StdlibConstantValue::String("Trailer:")
    );
    let default_max_header_bytes = resolve_stdlib_constant("net/http", "DefaultMaxHeaderBytes")
        .expect("http.DefaultMaxHeaderBytes should exist");
    assert_eq!(default_max_header_bytes.typ, "int");
    assert_eq!(
        default_max_header_bytes.value,
        super::StdlibConstantValue::Int(1 << 20)
    );
    let err_missing_file = resolve_stdlib_value("net/http", "ErrMissingFile")
        .expect("http.ErrMissingFile should exist");
    assert_eq!(err_missing_file.typ, "error");
    assert_eq!(
        err_missing_file.value,
        StdlibValueInit::Constant(super::StdlibConstantValue::Error("http: no such file"))
    );
    let err_no_cookie =
        resolve_stdlib_value("net/http", "ErrNoCookie").expect("http.ErrNoCookie should exist");
    assert_eq!(err_no_cookie.typ, "error");
    assert_eq!(
        err_no_cookie.value,
        StdlibValueInit::Constant(super::StdlibConstantValue::Error(
            "http: named cookie not present"
        ))
    );
    let err_no_location =
        resolve_stdlib_value("net/http", "ErrNoLocation").expect("http.ErrNoLocation should exist");
    assert_eq!(err_no_location.typ, "error");
    assert_eq!(
        err_no_location.value,
        StdlibValueInit::Constant(super::StdlibConstantValue::Error(
            "http: no Location header in response"
        ))
    );
    let err_use_last_response = resolve_stdlib_value("net/http", "ErrUseLastResponse")
        .expect("http.ErrUseLastResponse should exist");
    assert_eq!(err_use_last_response.typ, "error");
    assert_eq!(
        err_use_last_response.value,
        StdlibValueInit::Constant(super::StdlibConstantValue::Error(
            "net/http: use last response"
        ))
    );
    let err_abort_handler = resolve_stdlib_value("net/http", "ErrAbortHandler")
        .expect("http.ErrAbortHandler should exist");
    assert_eq!(err_abort_handler.typ, "error");
    assert_eq!(
        err_abort_handler.value,
        StdlibValueInit::Constant(super::StdlibConstantValue::Error("net/http: abort Handler"))
    );
    let err_server_closed = resolve_stdlib_value("net/http", "ErrServerClosed")
        .expect("http.ErrServerClosed should exist");
    assert_eq!(err_server_closed.typ, "error");
    assert_eq!(
        err_server_closed.value,
        StdlibValueInit::Constant(super::StdlibConstantValue::Error("http: Server closed"))
    );
    let default_client =
        resolve_stdlib_value("net/http", "DefaultClient").expect("http.DefaultClient should exist");
    assert_eq!(default_client.typ, "*http.Client");
    assert_eq!(
        default_client.value,
        StdlibValueInit::NewPointer("http.Client")
    );
    let canonical_header_key = resolve_stdlib_function("net/http", "CanonicalHeaderKey")
        .expect("http.CanonicalHeaderKey should exist");
    assert!(stdlib_function_returns_value(canonical_header_key));
    assert_eq!(stdlib_function_result_count(canonical_header_key), 1);
    assert_eq!(
        stdlib_function_param_types(canonical_header_key),
        Some(&["string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(canonical_header_key),
        Some(&["string"][..])
    );
    let status_text =
        resolve_stdlib_function("net/http", "StatusText").expect("http.StatusText should exist");
    assert!(stdlib_function_returns_value(status_text));
    assert_eq!(stdlib_function_result_count(status_text), 1);
    assert_eq!(stdlib_function_param_types(status_text), Some(&["int"][..]));
    assert_eq!(
        stdlib_function_result_types(status_text),
        Some(&["string"][..])
    );
    let parse_http_version = resolve_stdlib_function("net/http", "ParseHTTPVersion")
        .expect("http.ParseHTTPVersion should exist");
    assert!(!stdlib_function_returns_value(parse_http_version));
    assert_eq!(stdlib_function_result_count(parse_http_version), 3);
    assert_eq!(
        stdlib_function_param_types(parse_http_version),
        Some(&["string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(parse_http_version),
        Some(&["int", "int", "bool"][..])
    );
    let parse_time =
        resolve_stdlib_function("net/http", "ParseTime").expect("http.ParseTime should exist");
    assert!(!stdlib_function_returns_value(parse_time));
    assert_eq!(stdlib_function_result_count(parse_time), 2);
    assert_eq!(
        stdlib_function_param_types(parse_time),
        Some(&["string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(parse_time),
        Some(&["time.Time", "error"][..])
    );
    let detect_content_type = resolve_stdlib_function("net/http", "DetectContentType")
        .expect("http.DetectContentType should exist");
    assert!(stdlib_function_returns_value(detect_content_type));
    assert_eq!(stdlib_function_result_count(detect_content_type), 1);
    assert_eq!(
        stdlib_function_param_types(detect_content_type),
        Some(&["[]byte"][..])
    );
    assert_eq!(
        stdlib_function_result_types(detect_content_type),
        Some(&["string"][..])
    );
    let ints_are_sorted =
        resolve_stdlib_function("sort", "IntsAreSorted").expect("sort.IntsAreSorted should exist");
    assert!(stdlib_function_returns_value(ints_are_sorted));
    assert_eq!(stdlib_function_result_count(ints_are_sorted), 1);
    assert_eq!(
        stdlib_function_result_types(ints_are_sorted),
        Some(&["bool"][..])
    );
    let search_ints =
        resolve_stdlib_function("sort", "SearchInts").expect("sort.SearchInts should exist");
    assert!(stdlib_function_returns_value(search_ints));
    assert_eq!(stdlib_function_result_count(search_ints), 1);
    assert_eq!(
        stdlib_function_result_types(search_ints),
        Some(&["int"][..])
    );
}

#[test]
fn resolves_builtin_len_from_the_registry() {
    let function = resolve_stdlib_function("builtin", "len").expect("len should exist");
    assert!(stdlib_function_returns_value(function));
    let append = resolve_stdlib_function("builtin", "append").expect("append should exist");
    assert!(stdlib_function_returns_value(append));
    let range_keys =
        resolve_stdlib_function("builtin", "__range_keys").expect("range keys should exist");
    assert!(stdlib_function_returns_value(range_keys));
    let cap = resolve_stdlib_function("builtin", "cap").expect("cap should exist");
    assert!(stdlib_function_returns_value(cap));
    let range_value =
        resolve_stdlib_function("builtin", "__range_value").expect("range value should exist");
    assert!(stdlib_function_returns_value(range_value));
    let delete = resolve_stdlib_function("builtin", "delete").expect("delete should exist");
    assert!(stdlib_function_returns_value(delete));
    let make_slice =
        resolve_stdlib_function("builtin", "__make_slice").expect("make slice should exist");
    assert!(stdlib_function_returns_value(make_slice));
    assert!(resolve_stdlib_function("builtin", "copy").is_none());
    let contains = resolve_stdlib_function("strings", "Contains").expect("Contains should exist");
    assert!(stdlib_function_returns_value(contains));
    let trim_space =
        resolve_stdlib_function("strings", "TrimSpace").expect("TrimSpace should exist");
    assert!(stdlib_function_returns_value(trim_space));
    let count = resolve_stdlib_function("strings", "Count").expect("Count should exist");
    assert!(stdlib_function_returns_value(count));
    let repeat = resolve_stdlib_function("strings", "Repeat").expect("Repeat should exist");
    assert!(stdlib_function_returns_value(repeat));
    let split = resolve_stdlib_function("strings", "Split").expect("Split should exist");
    assert!(stdlib_function_returns_value(split));
    let join = resolve_stdlib_function("strings", "Join").expect("Join should exist");
    assert!(stdlib_function_returns_value(join));
    let replace_all =
        resolve_stdlib_function("strings", "ReplaceAll").expect("ReplaceAll should exist");
    assert!(stdlib_function_returns_value(replace_all));
    let fields = resolve_stdlib_function("strings", "Fields").expect("Fields should exist");
    assert!(stdlib_function_returns_value(fields));
    let index = resolve_stdlib_function("strings", "Index").expect("Index should exist");
    assert!(stdlib_function_returns_value(index));
    let trim_prefix =
        resolve_stdlib_function("strings", "TrimPrefix").expect("TrimPrefix should exist");
    assert!(stdlib_function_returns_value(trim_prefix));
    let trim_suffix =
        resolve_stdlib_function("strings", "TrimSuffix").expect("TrimSuffix should exist");
    assert!(stdlib_function_returns_value(trim_suffix));
    let last_index =
        resolve_stdlib_function("strings", "LastIndex").expect("LastIndex should exist");
    assert!(stdlib_function_returns_value(last_index));
    let trim_left = resolve_stdlib_function("strings", "TrimLeft").expect("TrimLeft should exist");
    assert!(stdlib_function_returns_value(trim_left));
    let trim_right =
        resolve_stdlib_function("strings", "TrimRight").expect("TrimRight should exist");
    assert!(stdlib_function_returns_value(trim_right));
    let trim = resolve_stdlib_function("strings", "Trim").expect("Trim should exist");
    assert!(stdlib_function_returns_value(trim));
    let contains_any =
        resolve_stdlib_function("strings", "ContainsAny").expect("ContainsAny should exist");
    assert!(stdlib_function_returns_value(contains_any));
    let index_any = resolve_stdlib_function("strings", "IndexAny").expect("IndexAny should exist");
    assert!(stdlib_function_returns_value(index_any));
    let last_index_any =
        resolve_stdlib_function("strings", "LastIndexAny").expect("LastIndexAny should exist");
    assert!(stdlib_function_returns_value(last_index_any));
    let clone = resolve_stdlib_function("strings", "Clone").expect("Clone should exist");
    assert!(stdlib_function_returns_value(clone));
    let contains_rune =
        resolve_stdlib_function("strings", "ContainsRune").expect("ContainsRune should exist");
    assert!(stdlib_function_returns_value(contains_rune));
    let index_rune =
        resolve_stdlib_function("strings", "IndexRune").expect("IndexRune should exist");
    assert!(stdlib_function_returns_value(index_rune));
    let compare = resolve_stdlib_function("strings", "Compare").expect("Compare should exist");
    assert!(stdlib_function_returns_value(compare));
    let replace = resolve_stdlib_function("strings", "Replace").expect("Replace should exist");
    assert!(stdlib_function_returns_value(replace));
    let index_byte =
        resolve_stdlib_function("strings", "IndexByte").expect("IndexByte should exist");
    assert!(stdlib_function_returns_value(index_byte));
    let last_index_byte =
        resolve_stdlib_function("strings", "LastIndexByte").expect("LastIndexByte should exist");
    assert!(stdlib_function_returns_value(last_index_byte));
    let cut_prefix =
        resolve_stdlib_function("strings", "CutPrefix").expect("CutPrefix should exist");
    assert!(!stdlib_function_returns_value(cut_prefix));
    assert_eq!(stdlib_function_result_count(cut_prefix), 2);
    let cut_suffix =
        resolve_stdlib_function("strings", "CutSuffix").expect("CutSuffix should exist");
    assert!(!stdlib_function_returns_value(cut_suffix));
    assert_eq!(stdlib_function_result_count(cut_suffix), 2);
    let cut = resolve_stdlib_function("strings", "Cut").expect("Cut should exist");
    assert!(!stdlib_function_returns_value(cut));
    assert_eq!(stdlib_function_result_count(cut), 3);
    let equal_fold =
        resolve_stdlib_function("strings", "EqualFold").expect("EqualFold should exist");
    assert!(stdlib_function_returns_value(equal_fold));
    assert_eq!(stdlib_function_result_count(equal_fold), 1);
    assert_eq!(
        stdlib_function_result_types(equal_fold),
        Some(&["bool"][..])
    );
    let split_n = resolve_stdlib_function("strings", "SplitN").expect("SplitN should exist");
    assert!(stdlib_function_returns_value(split_n));
    assert_eq!(stdlib_function_result_count(split_n), 1);
    assert_eq!(
        stdlib_function_result_types(split_n),
        Some(&["[]string"][..])
    );
    let split_after_n =
        resolve_stdlib_function("strings", "SplitAfterN").expect("SplitAfterN should exist");
    assert!(stdlib_function_returns_value(split_after_n));
    assert_eq!(stdlib_function_result_count(split_after_n), 1);
    assert_eq!(
        stdlib_function_result_types(split_after_n),
        Some(&["[]string"][..])
    );
    let atoi = resolve_stdlib_function("strconv", "Atoi").expect("Atoi should exist");
    assert!(!stdlib_function_returns_value(atoi));
    assert_eq!(stdlib_function_result_count(atoi), 2);
    assert_eq!(
        stdlib_function_result_types(atoi),
        Some(&["int", "error"][..])
    );
    let new = resolve_stdlib_function("errors", "New").expect("errors.New should exist");
    assert!(stdlib_function_returns_value(new));
    assert_eq!(stdlib_function_result_count(new), 1);
    assert_eq!(stdlib_function_result_types(new), Some(&["error"][..]));
    let join = resolve_stdlib_function("errors", "Join").expect("errors.Join should exist");
    assert!(stdlib_function_returns_value(join));
    assert_eq!(stdlib_function_result_count(join), 1);
    assert_eq!(stdlib_function_result_types(join), Some(&["error"][..]));
    let parse_bool =
        resolve_stdlib_function("strconv", "ParseBool").expect("ParseBool should exist");
    assert!(!stdlib_function_returns_value(parse_bool));
    assert_eq!(stdlib_function_result_count(parse_bool), 2);
    assert_eq!(
        stdlib_function_result_types(parse_bool),
        Some(&["bool", "error"][..])
    );
    let parse_int = resolve_stdlib_function("strconv", "ParseInt").expect("ParseInt should exist");
    assert!(!stdlib_function_returns_value(parse_int));
    assert_eq!(stdlib_function_result_count(parse_int), 2);
    assert_eq!(
        stdlib_function_result_types(parse_int),
        Some(&["int", "error"][..])
    );
    let unquote = resolve_stdlib_function("strconv", "Unquote").expect("Unquote should exist");
    assert!(!stdlib_function_returns_value(unquote));
    assert_eq!(stdlib_function_result_count(unquote), 2);
    assert_eq!(
        stdlib_function_result_types(unquote),
        Some(&["string", "error"][..])
    );
    let unquote_char =
        resolve_stdlib_function("strconv", "UnquoteChar").expect("UnquoteChar should exist");
    assert!(!stdlib_function_returns_value(unquote_char));
    assert_eq!(stdlib_function_result_count(unquote_char), 4);
    assert_eq!(
        stdlib_function_result_types(unquote_char),
        Some(&["int", "bool", "string", "error"][..])
    );
    let itoa = resolve_stdlib_function("strconv", "Itoa").expect("Itoa should exist");
    assert!(stdlib_function_returns_value(itoa));
    let can_backquote =
        resolve_stdlib_function("strconv", "CanBackquote").expect("CanBackquote should exist");
    assert!(stdlib_function_returns_value(can_backquote));
    let format_int =
        resolve_stdlib_function("strconv", "FormatInt").expect("FormatInt should exist");
    assert!(stdlib_function_returns_value(format_int));
    let quote_to_ascii =
        resolve_stdlib_function("strconv", "QuoteToASCII").expect("QuoteToASCII should exist");
    assert!(stdlib_function_returns_value(quote_to_ascii));
    let quote_rune_to_ascii = resolve_stdlib_function("strconv", "QuoteRuneToASCII")
        .expect("QuoteRuneToASCII should exist");
    assert!(stdlib_function_returns_value(quote_rune_to_ascii));
    let quote_rune =
        resolve_stdlib_function("strconv", "QuoteRune").expect("QuoteRune should exist");
    assert!(stdlib_function_returns_value(quote_rune));
    let parse_uint =
        resolve_stdlib_function("strconv", "ParseUint").expect("ParseUint should exist");
    assert!(!stdlib_function_returns_value(parse_uint));
    assert_eq!(stdlib_function_result_count(parse_uint), 2);
    assert_eq!(
        stdlib_function_result_types(parse_uint),
        Some(&["int", "error"][..])
    );
    let is_digit = resolve_stdlib_function("unicode", "IsDigit").expect("IsDigit should exist");
    assert!(stdlib_function_returns_value(is_digit));
    let is_letter = resolve_stdlib_function("unicode", "IsLetter").expect("IsLetter should exist");
    assert!(stdlib_function_returns_value(is_letter));
    let is_space = resolve_stdlib_function("unicode", "IsSpace").expect("IsSpace should exist");
    assert!(stdlib_function_returns_value(is_space));
    let is_upper = resolve_stdlib_function("unicode", "IsUpper").expect("IsUpper should exist");
    assert!(stdlib_function_returns_value(is_upper));
    let is_lower = resolve_stdlib_function("unicode", "IsLower").expect("IsLower should exist");
    assert!(stdlib_function_returns_value(is_lower));
    let is_number = resolve_stdlib_function("unicode", "IsNumber").expect("IsNumber should exist");
    assert!(stdlib_function_returns_value(is_number));
    let is_print = resolve_stdlib_function("unicode", "IsPrint").expect("IsPrint should exist");
    assert!(stdlib_function_returns_value(is_print));
    let is_graphic =
        resolve_stdlib_function("unicode", "IsGraphic").expect("IsGraphic should exist");
    assert!(stdlib_function_returns_value(is_graphic));
    let is_punct = resolve_stdlib_function("unicode", "IsPunct").expect("IsPunct should exist");
    assert!(stdlib_function_returns_value(is_punct));
    let is_symbol = resolve_stdlib_function("unicode", "IsSymbol").expect("IsSymbol should exist");
    assert!(stdlib_function_returns_value(is_symbol));
    let is_mark = resolve_stdlib_function("unicode", "IsMark").expect("IsMark should exist");
    assert!(stdlib_function_returns_value(is_mark));
    let is_title = resolve_stdlib_function("unicode", "IsTitle").expect("IsTitle should exist");
    assert!(stdlib_function_returns_value(is_title));
    let is_control =
        resolve_stdlib_function("unicode", "IsControl").expect("IsControl should exist");
    assert!(stdlib_function_returns_value(is_control));
    let to_upper = resolve_stdlib_function("unicode", "ToUpper").expect("ToUpper should exist");
    assert!(stdlib_function_returns_value(to_upper));
    let to_lower = resolve_stdlib_function("unicode", "ToLower").expect("ToLower should exist");
    assert!(stdlib_function_returns_value(to_lower));
    let to_title = resolve_stdlib_function("unicode", "ToTitle").expect("ToTitle should exist");
    assert!(stdlib_function_returns_value(to_title));
    let to = resolve_stdlib_function("unicode", "To").expect("To should exist");
    assert!(stdlib_function_returns_value(to));
    let simple_fold =
        resolve_stdlib_function("unicode", "SimpleFold").expect("SimpleFold should exist");
    assert!(stdlib_function_returns_value(simple_fold));
}

#[test]
fn exposes_stdlib_parameter_type_metadata() {
    let contains = resolve_stdlib_function("strings", "Contains").expect("Contains should exist");
    assert_eq!(
        stdlib_function_param_types(contains),
        Some(&["string", "string"][..])
    );

    let join = resolve_stdlib_function("strings", "Join").expect("Join should exist");
    assert_eq!(
        stdlib_function_param_types(join),
        Some(&["[]string", "string"][..])
    );

    let itoa = resolve_stdlib_function("strconv", "Itoa").expect("Itoa should exist");
    assert_eq!(stdlib_function_param_types(itoa), Some(&["int"][..]));

    let println = resolve_stdlib_function("fmt", "Println").expect("Println should exist");
    assert_eq!(stdlib_function_param_types(println), None);

    let join = resolve_stdlib_function("errors", "Join").expect("Join should exist");
    assert_eq!(stdlib_function_param_types(join), Some(&[][..]));
    assert_eq!(stdlib_function_variadic_param_type(join), Some("error"));

    let clean = resolve_stdlib_function("path", "Clean").expect("Clean should exist");
    assert_eq!(stdlib_function_param_types(clean), Some(&["string"][..]));

    let split = resolve_stdlib_function("path", "Split").expect("Split should exist");
    assert_eq!(stdlib_function_param_types(split), Some(&["string"][..]));

    let join = resolve_stdlib_function("path", "Join").expect("Join should exist");
    assert_eq!(stdlib_function_param_types(join), Some(&[][..]));
    assert_eq!(stdlib_function_variadic_param_type(join), Some("string"));

    let path_match = resolve_stdlib_function("path", "Match").expect("Match should exist");
    assert_eq!(
        stdlib_function_param_types(path_match),
        Some(&["string", "string"][..])
    );

    let ints_are_sorted =
        resolve_stdlib_function("sort", "IntsAreSorted").expect("IntsAreSorted should exist");
    assert_eq!(
        stdlib_function_param_types(ints_are_sorted),
        Some(&["[]int"][..])
    );

    let strings_are_sorted =
        resolve_stdlib_function("sort", "StringsAreSorted").expect("StringsAreSorted should exist");
    assert_eq!(
        stdlib_function_param_types(strings_are_sorted),
        Some(&["[]string"][..])
    );

    let search_ints =
        resolve_stdlib_function("sort", "SearchInts").expect("SearchInts should exist");
    assert_eq!(
        stdlib_function_param_types(search_ints),
        Some(&["[]int", "int"][..])
    );

    let search_strings =
        resolve_stdlib_function("sort", "SearchStrings").expect("SearchStrings should exist");
    assert_eq!(
        stdlib_function_param_types(search_strings),
        Some(&["[]string", "string"][..])
    );

    let sort_ints = resolve_stdlib_function("sort", "Ints").expect("Ints should exist");
    assert_eq!(stdlib_function_param_types(sort_ints), Some(&["[]int"][..]));

    let sort_strings = resolve_stdlib_function("sort", "Strings").expect("Strings should exist");
    assert_eq!(
        stdlib_function_param_types(sort_strings),
        Some(&["[]string"][..])
    );

    let sort_float64s = resolve_stdlib_function("sort", "Float64s").expect("Float64s should exist");
    assert_eq!(
        stdlib_function_param_types(sort_float64s),
        Some(&["[]float64"][..])
    );

    let is_digit = resolve_stdlib_function("unicode", "IsDigit").expect("IsDigit should exist");
    assert_eq!(stdlib_function_param_types(is_digit), Some(&["int"][..]));

    let to_upper = resolve_stdlib_function("unicode", "ToUpper").expect("ToUpper should exist");
    assert_eq!(stdlib_function_param_types(to_upper), Some(&["int"][..]));

    let is_print = resolve_stdlib_function("unicode", "IsPrint").expect("IsPrint should exist");
    assert_eq!(stdlib_function_param_types(is_print), Some(&["int"][..]));

    let is_graphic =
        resolve_stdlib_function("unicode", "IsGraphic").expect("IsGraphic should exist");
    assert_eq!(stdlib_function_param_types(is_graphic), Some(&["int"][..]));

    let is_punct = resolve_stdlib_function("unicode", "IsPunct").expect("IsPunct should exist");
    assert_eq!(stdlib_function_param_types(is_punct), Some(&["int"][..]));

    let is_symbol = resolve_stdlib_function("unicode", "IsSymbol").expect("IsSymbol should exist");
    assert_eq!(stdlib_function_param_types(is_symbol), Some(&["int"][..]));

    let is_mark = resolve_stdlib_function("unicode", "IsMark").expect("IsMark should exist");
    assert_eq!(stdlib_function_param_types(is_mark), Some(&["int"][..]));

    let is_title = resolve_stdlib_function("unicode", "IsTitle").expect("IsTitle should exist");
    assert_eq!(stdlib_function_param_types(is_title), Some(&["int"][..]));

    let is_control =
        resolve_stdlib_function("unicode", "IsControl").expect("IsControl should exist");
    assert_eq!(stdlib_function_param_types(is_control), Some(&["int"][..]));

    let to_title = resolve_stdlib_function("unicode", "ToTitle").expect("ToTitle should exist");
    assert_eq!(stdlib_function_param_types(to_title), Some(&["int"][..]));

    let to = resolve_stdlib_function("unicode", "To").expect("To should exist");
    assert_eq!(stdlib_function_param_types(to), Some(&["int", "int"][..]));

    let simple_fold =
        resolve_stdlib_function("unicode", "SimpleFold").expect("SimpleFold should exist");
    assert_eq!(stdlib_function_param_types(simple_fold), Some(&["int"][..]));

    let equal_fold =
        resolve_stdlib_function("strings", "EqualFold").expect("EqualFold should exist");
    assert_eq!(
        stdlib_function_param_types(equal_fold),
        Some(&["string", "string"][..])
    );

    let split_n = resolve_stdlib_function("strings", "SplitN").expect("SplitN should exist");
    assert_eq!(
        stdlib_function_param_types(split_n),
        Some(&["string", "string", "int"][..])
    );
    let split_after_n =
        resolve_stdlib_function("strings", "SplitAfterN").expect("SplitAfterN should exist");
    assert_eq!(
        stdlib_function_param_types(split_after_n),
        Some(&["string", "string", "int"][..])
    );
}

#[test]
fn resolves_unicode_constants_from_the_registry() {
    let version = resolve_stdlib_constant("unicode", "Version").expect("Version should exist");
    assert_eq!(version.typ, "string");
    assert_eq!(version.value, super::StdlibConstantValue::String("15.0.0"));

    let max_rune = resolve_stdlib_constant("unicode", "MaxRune").expect("MaxRune should exist");
    assert_eq!(max_rune.typ, "int");
    assert_eq!(max_rune.value, super::StdlibConstantValue::Int(0x10_FFFF));

    let replacement = resolve_stdlib_constant("unicode", "ReplacementChar")
        .expect("ReplacementChar should exist");
    assert_eq!(replacement.typ, "int");
    assert_eq!(replacement.value, super::StdlibConstantValue::Int(0xFFFD));

    let max_ascii = resolve_stdlib_constant("unicode", "MaxASCII").expect("MaxASCII should exist");
    assert_eq!(max_ascii.typ, "int");
    assert_eq!(max_ascii.value, super::StdlibConstantValue::Int(0x7F));

    let max_latin1 =
        resolve_stdlib_constant("unicode", "MaxLatin1").expect("MaxLatin1 should exist");
    assert_eq!(max_latin1.typ, "int");
    assert_eq!(max_latin1.value, super::StdlibConstantValue::Int(0xFF));

    let upper = resolve_stdlib_constant("unicode", "UpperCase").expect("UpperCase should exist");
    assert_eq!(upper.typ, "int");
    assert_eq!(upper.value, super::StdlibConstantValue::Int(0));

    let lower = resolve_stdlib_constant("unicode", "LowerCase").expect("LowerCase should exist");
    assert_eq!(lower.typ, "int");
    assert_eq!(lower.value, super::StdlibConstantValue::Int(1));

    let title = resolve_stdlib_constant("unicode", "TitleCase").expect("TitleCase should exist");
    assert_eq!(title.typ, "int");
    assert_eq!(title.value, super::StdlibConstantValue::Int(2));
}

#[test]
fn resolves_net_http_constants_from_the_registry() {
    let method_get = resolve_stdlib_constant("net/http", "MethodGet").expect("MethodGet exists");
    assert_eq!(method_get.typ, "string");
    assert_eq!(method_get.value, super::StdlibConstantValue::String("GET"));

    let method_patch =
        resolve_stdlib_constant("net/http", "MethodPatch").expect("MethodPatch exists");
    assert_eq!(method_patch.typ, "string");
    assert_eq!(
        method_patch.value,
        super::StdlibConstantValue::String("PATCH")
    );

    let method_trace =
        resolve_stdlib_constant("net/http", "MethodTrace").expect("MethodTrace exists");
    assert_eq!(method_trace.typ, "string");
    assert_eq!(
        method_trace.value,
        super::StdlibConstantValue::String("TRACE")
    );

    let status_ok = resolve_stdlib_constant("net/http", "StatusOK").expect("StatusOK exists");
    assert_eq!(status_ok.typ, "int");
    assert_eq!(status_ok.value, super::StdlibConstantValue::Int(200));

    let status_teapot =
        resolve_stdlib_constant("net/http", "StatusTeapot").expect("StatusTeapot exists");
    assert_eq!(status_teapot.typ, "int");
    assert_eq!(status_teapot.value, super::StdlibConstantValue::Int(418));
}
