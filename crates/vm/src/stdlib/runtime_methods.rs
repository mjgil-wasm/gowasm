use super::{
    base64_impl, context_impl, io_fs_registry_impl, net_http_impl, net_url_impl, reflect_impl,
    regexp_impl, strings_replacer_impl, sync_impl, time_impl, StdlibFunctionId,
};
use crate::{
    TypeId, TYPE_BASE64_ENCODING_PTR, TYPE_CONTEXT, TYPE_FS_DIR_ENTRY, TYPE_FS_FILE,
    TYPE_FS_FILE_INFO, TYPE_FS_FILE_MODE, TYPE_FS_SUB_FS, TYPE_HTTP_CLIENT_PTR, TYPE_HTTP_HEADER,
    TYPE_HTTP_REQUEST_BODY, TYPE_HTTP_REQUEST_PTR, TYPE_HTTP_RESPONSE_BODY, TYPE_OS_DIR_FS,
    TYPE_REFLECT_KIND, TYPE_REFLECT_RTYPE, TYPE_REFLECT_RVALUE, TYPE_REFLECT_STRUCT_TAG,
    TYPE_REGEXP, TYPE_STRINGS_REPLACER, TYPE_SYNC_MUTEX_PTR, TYPE_SYNC_ONCE_PTR,
    TYPE_SYNC_RW_MUTEX_PTR, TYPE_SYNC_WAIT_GROUP_PTR, TYPE_TIME, TYPE_TIME_DURATION, TYPE_TIME_PTR,
    TYPE_TIME_TIMER_PTR, TYPE_URL, TYPE_URL_PTR, TYPE_URL_USERINFO_PTR, TYPE_URL_VALUES,
};

pub fn resolve_stdlib_method(receiver_type: &str, method: &str) -> Option<StdlibFunctionId> {
    regexp_impl::REGEXP_METHODS
        .iter()
        .chain(base64_impl::BASE64_METHODS.iter())
        .chain(context_impl::CONTEXT_METHODS.iter())
        .chain(io_fs_registry_impl::IO_FS_METHODS.iter())
        .chain(net_http_impl::NET_HTTP_METHODS.iter())
        .chain(net_http_impl::NET_HTTP_REQUEST_METHODS.iter())
        .chain(net_http_impl::NET_HTTP_REQUEST_BODY_METHODS.iter())
        .chain(net_http_impl::NET_HTTP_RESPONSE_METHODS.iter())
        .chain(net_http_impl::NET_HTTP_TRANSPORT_METHODS.iter())
        .chain(net_url_impl::NET_URL_METHODS.iter())
        .chain(reflect_impl::REFLECT_METHODS.iter())
        .chain(strings_replacer_impl::STRINGS_REPLACER_METHODS.iter())
        .chain(sync_impl::SYNC_METHODS.iter())
        .chain(time_impl::TIME_METHODS.iter())
        .find(|entry| entry.receiver_type == receiver_type && entry.method == method)
        .map(|entry| entry.function)
}

pub fn resolve_stdlib_runtime_method(
    receiver_type: TypeId,
    method: &str,
) -> Option<StdlibFunctionId> {
    runtime_stdlib_receiver_types(receiver_type)
        .iter()
        .find_map(|receiver_type| resolve_stdlib_method(receiver_type, method))
}

fn runtime_stdlib_receiver_types(receiver_type: TypeId) -> &'static [&'static str] {
    match receiver_type {
        TYPE_BASE64_ENCODING_PTR => &["*base64.Encoding"],
        TYPE_CONTEXT => &["context.__impl"],
        TYPE_FS_DIR_ENTRY => &["fs.DirEntry"],
        TYPE_FS_FILE => &["fs.ReadDirFile", "fs.File"],
        TYPE_FS_FILE_INFO => &["fs.FileInfo"],
        TYPE_FS_FILE_MODE => &["fs.FileMode"],
        TYPE_FS_SUB_FS => &["fs.FS"],
        TYPE_HTTP_CLIENT_PTR => &["*http.Client"],
        TYPE_HTTP_HEADER => &["http.Header"],
        TYPE_HTTP_REQUEST_BODY => &["http.__requestBody"],
        TYPE_HTTP_REQUEST_PTR => &["*http.Request"],
        TYPE_HTTP_RESPONSE_BODY => &["http.__responseBody"],
        TYPE_OS_DIR_FS => &["fs.FS"],
        TYPE_REFLECT_KIND => &["reflect.Kind"],
        TYPE_REFLECT_RTYPE => &["reflect.__type"],
        TYPE_REFLECT_RVALUE => &["reflect.__value"],
        TYPE_REFLECT_STRUCT_TAG => &["reflect.StructTag"],
        TYPE_REGEXP => &["*regexp.Regexp"],
        TYPE_STRINGS_REPLACER => &["*strings.Replacer"],
        TYPE_SYNC_WAIT_GROUP_PTR => &["*sync.WaitGroup"],
        TYPE_SYNC_ONCE_PTR => &["*sync.Once"],
        TYPE_SYNC_MUTEX_PTR => &["*sync.Mutex"],
        TYPE_SYNC_RW_MUTEX_PTR => &["*sync.RWMutex"],
        TYPE_TIME => &["time.Time"],
        TYPE_TIME_PTR => &["time.Time"],
        TYPE_TIME_TIMER_PTR => &["*time.Timer"],
        TYPE_TIME_DURATION => &["time.Duration"],
        TYPE_URL_VALUES => &["url.Values"],
        TYPE_URL => &["url.URL"],
        TYPE_URL_PTR => &["*url.URL", "url.URL"],
        TYPE_URL_USERINFO_PTR => &["*url.Userinfo"],
        _ => &[],
    }
}
