mod core;
mod walk;

use super::{
    StdlibConstantValue, StdlibFunction, StdlibValue, StdlibValueInit, FILEPATH_ABS, FILEPATH_BASE,
    FILEPATH_CLEAN, FILEPATH_DIR, FILEPATH_EXT, FILEPATH_FROM_SLASH, FILEPATH_GLOB,
    FILEPATH_IS_ABS, FILEPATH_IS_LOCAL, FILEPATH_JOIN, FILEPATH_LOCALIZE, FILEPATH_MATCH,
    FILEPATH_REL, FILEPATH_SPLIT, FILEPATH_SPLIT_LIST, FILEPATH_TO_SLASH, FILEPATH_VOLUME_NAME,
    FILEPATH_WALK, FILEPATH_WALK_DIR,
};
use crate::{Program, Value, Vm, VmError};

pub(super) use self::core::{
    filepath_abs, filepath_base, filepath_clean, filepath_dir, filepath_ext, filepath_from_slash,
    filepath_glob, filepath_is_abs, filepath_is_local, filepath_join, filepath_localize,
    filepath_match, filepath_rel, filepath_split, filepath_split_list, filepath_to_slash,
    filepath_volume_name,
};
pub(super) use self::walk::{filepath_walk, filepath_walk_dir};

pub(super) const FILEPATH_VALUES: &[StdlibValue] = &[
    StdlibValue {
        symbol: "SkipDir",
        typ: "error",
        value: StdlibValueInit::Constant(StdlibConstantValue::Error(
            super::io_fs_registry_impl::SKIP_DIR,
        )),
    },
    StdlibValue {
        symbol: "SkipAll",
        typ: "error",
        value: StdlibValueInit::Constant(StdlibConstantValue::Error(
            super::io_fs_registry_impl::SKIP_ALL,
        )),
    },
];

pub(super) const FILEPATH_FUNCTIONS: &[StdlibFunction] = &[
    StdlibFunction {
        id: FILEPATH_BASE,
        symbol: "Base",
        returns_value: true,
        handler: filepath_base,
    },
    StdlibFunction {
        id: FILEPATH_CLEAN,
        symbol: "Clean",
        returns_value: true,
        handler: filepath_clean,
    },
    StdlibFunction {
        id: FILEPATH_DIR,
        symbol: "Dir",
        returns_value: true,
        handler: filepath_dir,
    },
    StdlibFunction {
        id: FILEPATH_EXT,
        symbol: "Ext",
        returns_value: true,
        handler: filepath_ext,
    },
    StdlibFunction {
        id: FILEPATH_IS_ABS,
        symbol: "IsAbs",
        returns_value: true,
        handler: filepath_is_abs,
    },
    StdlibFunction {
        id: FILEPATH_SPLIT,
        symbol: "Split",
        returns_value: false,
        handler: unsupported_filepath_multi_result,
    },
    StdlibFunction {
        id: FILEPATH_JOIN,
        symbol: "Join",
        returns_value: true,
        handler: filepath_join,
    },
    StdlibFunction {
        id: FILEPATH_MATCH,
        symbol: "Match",
        returns_value: false,
        handler: unsupported_filepath_multi_result,
    },
    StdlibFunction {
        id: FILEPATH_TO_SLASH,
        symbol: "ToSlash",
        returns_value: true,
        handler: filepath_to_slash,
    },
    StdlibFunction {
        id: FILEPATH_FROM_SLASH,
        symbol: "FromSlash",
        returns_value: true,
        handler: filepath_from_slash,
    },
    StdlibFunction {
        id: FILEPATH_SPLIT_LIST,
        symbol: "SplitList",
        returns_value: true,
        handler: filepath_split_list,
    },
    StdlibFunction {
        id: FILEPATH_VOLUME_NAME,
        symbol: "VolumeName",
        returns_value: true,
        handler: filepath_volume_name,
    },
    StdlibFunction {
        id: FILEPATH_REL,
        symbol: "Rel",
        returns_value: false,
        handler: unsupported_filepath_multi_result,
    },
    StdlibFunction {
        id: FILEPATH_IS_LOCAL,
        symbol: "IsLocal",
        returns_value: true,
        handler: filepath_is_local,
    },
    StdlibFunction {
        id: FILEPATH_LOCALIZE,
        symbol: "Localize",
        returns_value: false,
        handler: unsupported_filepath_multi_result,
    },
    StdlibFunction {
        id: FILEPATH_GLOB,
        symbol: "Glob",
        returns_value: false,
        handler: unsupported_filepath_multi_result,
    },
    StdlibFunction {
        id: FILEPATH_WALK_DIR,
        symbol: "WalkDir",
        returns_value: true,
        handler: filepath_walk_dir,
    },
    StdlibFunction {
        id: FILEPATH_WALK,
        symbol: "Walk",
        returns_value: true,
        handler: filepath_walk,
    },
    StdlibFunction {
        id: FILEPATH_ABS,
        symbol: "Abs",
        returns_value: false,
        handler: unsupported_filepath_multi_result,
    },
];

fn unsupported_filepath_multi_result(
    _vm: &mut Vm,
    _program: &Program,
    _args: &[Value],
) -> Result<Value, VmError> {
    Ok(Value::nil())
}
