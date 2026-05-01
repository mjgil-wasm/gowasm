use std::collections::HashMap;

use gowasm_parser::{InterfaceMethodDecl, Parameter, TypeFieldDecl};
use gowasm_vm::{
    TypeId, TYPE_BASE64_ENCODING, TYPE_EMPTY_STRUCT, TYPE_HTTP_CLIENT, TYPE_HTTP_REQUEST,
    TYPE_HTTP_RESPONSE, TYPE_REGEXP, TYPE_STRINGS_REPLACER, TYPE_SYNC_MUTEX, TYPE_SYNC_ONCE,
    TYPE_SYNC_RW_MUTEX, TYPE_SYNC_WAIT_GROUP, TYPE_TIME, TYPE_TIME_TIMER, TYPE_URL,
    TYPE_URL_USERINFO,
};

use super::{InterfaceTypeDef, StructTypeDef};

#[path = "types_imported_reflect.rs"]
mod reflect_seed;

pub(super) const FIRST_USER_TYPE_ID: u32 = 115;

pub(crate) fn is_imported_type_only_package(path: &str) -> bool {
    matches!(path, "io")
}

pub(super) fn seed_imported_structs(structs: &mut HashMap<String, StructTypeDef>) {
    structs.extend([
        (
            "base64.Encoding".into(),
            StructTypeDef {
                type_id: TYPE_BASE64_ENCODING,
                fields: vec![TypeFieldDecl {
                    name: "__encodingKind".into(),
                    typ: "int".into(),
                    embedded: false,
                    tag: None,
                }],
            },
        ),
        (
            "sync.WaitGroup".into(),
            StructTypeDef {
                type_id: TYPE_SYNC_WAIT_GROUP,
                fields: Vec::new(),
            },
        ),
        (
            "regexp.Regexp".into(),
            StructTypeDef {
                type_id: TYPE_REGEXP,
                fields: vec![TypeFieldDecl {
                    name: "__regexp_id".into(),
                    typ: "int".into(),
                    embedded: false,
                    tag: None,
                }],
            },
        ),
        (
            "strings.Replacer".into(),
            StructTypeDef {
                type_id: TYPE_STRINGS_REPLACER,
                fields: vec![TypeFieldDecl {
                    name: "__strings_replacer_id".into(),
                    typ: "int".into(),
                    embedded: false,
                    tag: None,
                }],
            },
        ),
        (
            "sync.Once".into(),
            StructTypeDef {
                type_id: TYPE_SYNC_ONCE,
                fields: Vec::new(),
            },
        ),
        (
            "sync.Mutex".into(),
            StructTypeDef {
                type_id: TYPE_SYNC_MUTEX,
                fields: Vec::new(),
            },
        ),
        (
            "sync.RWMutex".into(),
            StructTypeDef {
                type_id: TYPE_SYNC_RW_MUTEX,
                fields: Vec::new(),
            },
        ),
        (
            "time.Time".into(),
            StructTypeDef {
                type_id: TYPE_TIME,
                fields: Vec::new(),
            },
        ),
        (
            "time.Timer".into(),
            StructTypeDef {
                type_id: TYPE_TIME_TIMER,
                fields: vec![TypeFieldDecl {
                    name: "C".into(),
                    typ: "<-chan time.Time".into(),
                    embedded: false,
                    tag: None,
                }],
            },
        ),
        (
            "http.Request".into(),
            StructTypeDef {
                type_id: TYPE_HTTP_REQUEST,
                fields: vec![
                    TypeFieldDecl {
                        name: "Method".into(),
                        typ: "string".into(),
                        embedded: false,
                        tag: None,
                    },
                    TypeFieldDecl {
                        name: "URL".into(),
                        typ: "*url.URL".into(),
                        embedded: false,
                        tag: None,
                    },
                    TypeFieldDecl {
                        name: "Header".into(),
                        typ: "http.Header".into(),
                        embedded: false,
                        tag: None,
                    },
                    TypeFieldDecl {
                        name: "Body".into(),
                        typ: "io.Reader".into(),
                        embedded: false,
                        tag: None,
                    },
                ],
            },
        ),
        (
            "http.Client".into(),
            StructTypeDef {
                type_id: TYPE_HTTP_CLIENT,
                fields: Vec::new(),
            },
        ),
        (
            "http.Response".into(),
            StructTypeDef {
                type_id: TYPE_HTTP_RESPONSE,
                fields: vec![
                    TypeFieldDecl {
                        name: "Status".into(),
                        typ: "string".into(),
                        embedded: false,
                        tag: None,
                    },
                    TypeFieldDecl {
                        name: "StatusCode".into(),
                        typ: "int".into(),
                        embedded: false,
                        tag: None,
                    },
                    TypeFieldDecl {
                        name: "Header".into(),
                        typ: "http.Header".into(),
                        embedded: false,
                        tag: None,
                    },
                    TypeFieldDecl {
                        name: "Body".into(),
                        typ: "io.ReadCloser".into(),
                        embedded: false,
                        tag: None,
                    },
                ],
            },
        ),
        (
            "url.URL".into(),
            StructTypeDef {
                type_id: TYPE_URL,
                fields: vec![
                    TypeFieldDecl {
                        name: "Scheme".into(),
                        typ: "string".into(),
                        embedded: false,
                        tag: None,
                    },
                    TypeFieldDecl {
                        name: "Opaque".into(),
                        typ: "string".into(),
                        embedded: false,
                        tag: None,
                    },
                    TypeFieldDecl {
                        name: "User".into(),
                        typ: "*url.Userinfo".into(),
                        embedded: false,
                        tag: None,
                    },
                    TypeFieldDecl {
                        name: "Host".into(),
                        typ: "string".into(),
                        embedded: false,
                        tag: None,
                    },
                    TypeFieldDecl {
                        name: "Path".into(),
                        typ: "string".into(),
                        embedded: false,
                        tag: None,
                    },
                    TypeFieldDecl {
                        name: "RawPath".into(),
                        typ: "string".into(),
                        embedded: false,
                        tag: None,
                    },
                    TypeFieldDecl {
                        name: "ForceQuery".into(),
                        typ: "bool".into(),
                        embedded: false,
                        tag: None,
                    },
                    TypeFieldDecl {
                        name: "RawQuery".into(),
                        typ: "string".into(),
                        embedded: false,
                        tag: None,
                    },
                    TypeFieldDecl {
                        name: "Fragment".into(),
                        typ: "string".into(),
                        embedded: false,
                        tag: None,
                    },
                    TypeFieldDecl {
                        name: "RawFragment".into(),
                        typ: "string".into(),
                        embedded: false,
                        tag: None,
                    },
                ],
            },
        ),
        (
            "url.Userinfo".into(),
            StructTypeDef {
                type_id: TYPE_URL_USERINFO,
                fields: Vec::new(),
            },
        ),
        (
            "struct{}".into(),
            StructTypeDef {
                type_id: TYPE_EMPTY_STRUCT,
                fields: Vec::new(),
            },
        ),
    ]);
    reflect_seed::extend_reflect_structs(structs);
}

pub(super) fn seed_imported_interfaces(interfaces: &mut HashMap<String, InterfaceTypeDef>) {
    interfaces.extend([
        (
            "context.Context".into(),
            InterfaceTypeDef {
                type_id: TypeId(100),
                methods: vec![
                    InterfaceMethodDecl {
                        name: "Deadline".into(),
                        params: Vec::new(),
                        result_types: vec!["time.Time".into(), "bool".into()],
                    },
                    InterfaceMethodDecl {
                        name: "Done".into(),
                        params: Vec::new(),
                        result_types: vec!["<-chan struct{}".into()],
                    },
                    InterfaceMethodDecl {
                        name: "Err".into(),
                        params: Vec::new(),
                        result_types: vec!["error".into()],
                    },
                    InterfaceMethodDecl {
                        name: "Value".into(),
                        params: vec![Parameter {
                            name: "key".into(),
                            typ: "interface{}".into(),
                            variadic: false,
                        }],
                        result_types: vec!["interface{}".into()],
                    },
                ],
            },
        ),
        (
            "fs.FS".into(),
            InterfaceTypeDef {
                type_id: TypeId(102),
                methods: vec![InterfaceMethodDecl {
                    name: "Open".into(),
                    params: vec![Parameter {
                        name: "name".into(),
                        typ: "string".into(),
                        variadic: false,
                    }],
                    result_types: vec!["fs.File".into(), "error".into()],
                }],
            },
        ),
        (
            "fs.File".into(),
            InterfaceTypeDef {
                type_id: TypeId(103),
                methods: vec![
                    InterfaceMethodDecl {
                        name: "Close".into(),
                        params: Vec::new(),
                        result_types: vec!["error".into()],
                    },
                    InterfaceMethodDecl {
                        name: "Stat".into(),
                        params: Vec::new(),
                        result_types: vec!["fs.FileInfo".into(), "error".into()],
                    },
                    InterfaceMethodDecl {
                        name: "Read".into(),
                        params: vec![Parameter {
                            name: "p".into(),
                            typ: "[]byte".into(),
                            variadic: false,
                        }],
                        result_types: vec!["int".into(), "error".into()],
                    },
                ],
            },
        ),
        (
            "io.Reader".into(),
            InterfaceTypeDef {
                type_id: TypeId(112),
                methods: vec![InterfaceMethodDecl {
                    name: "Read".into(),
                    params: vec![Parameter {
                        name: "p".into(),
                        typ: "[]byte".into(),
                        variadic: false,
                    }],
                    result_types: vec!["int".into(), "error".into()],
                }],
            },
        ),
        (
            "io.Closer".into(),
            InterfaceTypeDef {
                type_id: TypeId(113),
                methods: vec![InterfaceMethodDecl {
                    name: "Close".into(),
                    params: Vec::new(),
                    result_types: vec!["error".into()],
                }],
            },
        ),
        (
            "io.ReadCloser".into(),
            InterfaceTypeDef {
                type_id: TypeId(114),
                methods: vec![
                    InterfaceMethodDecl {
                        name: "Read".into(),
                        params: vec![Parameter {
                            name: "p".into(),
                            typ: "[]byte".into(),
                            variadic: false,
                        }],
                        result_types: vec!["int".into(), "error".into()],
                    },
                    InterfaceMethodDecl {
                        name: "Close".into(),
                        params: Vec::new(),
                        result_types: vec!["error".into()],
                    },
                ],
            },
        ),
        (
            "fs.FileInfo".into(),
            InterfaceTypeDef {
                type_id: TypeId(104),
                methods: vec![
                    InterfaceMethodDecl {
                        name: "Name".into(),
                        params: Vec::new(),
                        result_types: vec!["string".into()],
                    },
                    InterfaceMethodDecl {
                        name: "IsDir".into(),
                        params: Vec::new(),
                        result_types: vec!["bool".into()],
                    },
                    InterfaceMethodDecl {
                        name: "Size".into(),
                        params: Vec::new(),
                        result_types: vec!["int".into()],
                    },
                    InterfaceMethodDecl {
                        name: "Mode".into(),
                        params: Vec::new(),
                        result_types: vec!["fs.FileMode".into()],
                    },
                    InterfaceMethodDecl {
                        name: "ModTime".into(),
                        params: Vec::new(),
                        result_types: vec!["time.Time".into()],
                    },
                    InterfaceMethodDecl {
                        name: "Sys".into(),
                        params: Vec::new(),
                        result_types: vec!["interface{}".into()],
                    },
                ],
            },
        ),
        (
            "fs.DirEntry".into(),
            InterfaceTypeDef {
                type_id: TypeId(105),
                methods: vec![
                    InterfaceMethodDecl {
                        name: "Name".into(),
                        params: Vec::new(),
                        result_types: vec!["string".into()],
                    },
                    InterfaceMethodDecl {
                        name: "IsDir".into(),
                        params: Vec::new(),
                        result_types: vec!["bool".into()],
                    },
                    InterfaceMethodDecl {
                        name: "Type".into(),
                        params: Vec::new(),
                        result_types: vec!["fs.FileMode".into()],
                    },
                    InterfaceMethodDecl {
                        name: "Info".into(),
                        params: Vec::new(),
                        result_types: vec!["fs.FileInfo".into(), "error".into()],
                    },
                ],
            },
        ),
        (
            "fs.ReadDirFile".into(),
            InterfaceTypeDef {
                type_id: TypeId(106),
                methods: vec![
                    InterfaceMethodDecl {
                        name: "Close".into(),
                        params: Vec::new(),
                        result_types: vec!["error".into()],
                    },
                    InterfaceMethodDecl {
                        name: "Stat".into(),
                        params: Vec::new(),
                        result_types: vec!["fs.FileInfo".into(), "error".into()],
                    },
                    InterfaceMethodDecl {
                        name: "Read".into(),
                        params: vec![Parameter {
                            name: "p".into(),
                            typ: "[]byte".into(),
                            variadic: false,
                        }],
                        result_types: vec!["int".into(), "error".into()],
                    },
                    InterfaceMethodDecl {
                        name: "ReadDir".into(),
                        params: vec![Parameter {
                            name: "n".into(),
                            typ: "int".into(),
                            variadic: false,
                        }],
                        result_types: vec!["[]fs.DirEntry".into(), "error".into()],
                    },
                ],
            },
        ),
        (
            "fs.ReadFileFS".into(),
            InterfaceTypeDef {
                type_id: TypeId(107),
                methods: vec![
                    InterfaceMethodDecl {
                        name: "Open".into(),
                        params: vec![Parameter {
                            name: "name".into(),
                            typ: "string".into(),
                            variadic: false,
                        }],
                        result_types: vec!["fs.File".into(), "error".into()],
                    },
                    InterfaceMethodDecl {
                        name: "ReadFile".into(),
                        params: vec![Parameter {
                            name: "name".into(),
                            typ: "string".into(),
                            variadic: false,
                        }],
                        result_types: vec!["[]byte".into(), "error".into()],
                    },
                ],
            },
        ),
        (
            "fs.StatFS".into(),
            InterfaceTypeDef {
                type_id: TypeId(108),
                methods: vec![
                    InterfaceMethodDecl {
                        name: "Open".into(),
                        params: vec![Parameter {
                            name: "name".into(),
                            typ: "string".into(),
                            variadic: false,
                        }],
                        result_types: vec!["fs.File".into(), "error".into()],
                    },
                    InterfaceMethodDecl {
                        name: "Stat".into(),
                        params: vec![Parameter {
                            name: "name".into(),
                            typ: "string".into(),
                            variadic: false,
                        }],
                        result_types: vec!["fs.FileInfo".into(), "error".into()],
                    },
                ],
            },
        ),
        (
            "fs.ReadDirFS".into(),
            InterfaceTypeDef {
                type_id: TypeId(109),
                methods: vec![
                    InterfaceMethodDecl {
                        name: "Open".into(),
                        params: vec![Parameter {
                            name: "name".into(),
                            typ: "string".into(),
                            variadic: false,
                        }],
                        result_types: vec!["fs.File".into(), "error".into()],
                    },
                    InterfaceMethodDecl {
                        name: "ReadDir".into(),
                        params: vec![Parameter {
                            name: "name".into(),
                            typ: "string".into(),
                            variadic: false,
                        }],
                        result_types: vec!["[]fs.DirEntry".into(), "error".into()],
                    },
                ],
            },
        ),
        (
            "fs.GlobFS".into(),
            InterfaceTypeDef {
                type_id: TypeId(110),
                methods: vec![
                    InterfaceMethodDecl {
                        name: "Open".into(),
                        params: vec![Parameter {
                            name: "name".into(),
                            typ: "string".into(),
                            variadic: false,
                        }],
                        result_types: vec!["fs.File".into(), "error".into()],
                    },
                    InterfaceMethodDecl {
                        name: "Glob".into(),
                        params: vec![Parameter {
                            name: "pattern".into(),
                            typ: "string".into(),
                            variadic: false,
                        }],
                        result_types: vec!["[]string".into(), "error".into()],
                    },
                ],
            },
        ),
        (
            "fs.SubFS".into(),
            InterfaceTypeDef {
                type_id: TypeId(111),
                methods: vec![
                    InterfaceMethodDecl {
                        name: "Open".into(),
                        params: vec![Parameter {
                            name: "name".into(),
                            typ: "string".into(),
                            variadic: false,
                        }],
                        result_types: vec!["fs.File".into(), "error".into()],
                    },
                    InterfaceMethodDecl {
                        name: "Sub".into(),
                        params: vec![Parameter {
                            name: "dir".into(),
                            typ: "string".into(),
                            variadic: false,
                        }],
                        result_types: vec!["fs.FS".into(), "error".into()],
                    },
                ],
            },
        ),
    ]);
    reflect_seed::extend_reflect_interfaces(interfaces);
}
