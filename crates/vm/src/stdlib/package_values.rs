use super::{
    base64_impl, context_impl, filepath_impl, io_fs_registry_impl, net_http_impl, os_impl,
    StdlibValue,
};

pub fn resolve_stdlib_value(package: &str, symbol: &str) -> Option<&'static StdlibValue> {
    let values = match package {
        "encoding/base64" => base64_impl::BASE64_VALUES,
        "context" => context_impl::CONTEXT_VALUES,
        "io/fs" => io_fs_registry_impl::IO_FS_VALUES,
        "net/http" => net_http_impl::NET_HTTP_VALUES,
        "os" => os_impl::OS_VALUES,
        "path/filepath" => filepath_impl::FILEPATH_VALUES,
        _ => return None,
    };
    values.iter().find(|value| value.symbol == symbol)
}
