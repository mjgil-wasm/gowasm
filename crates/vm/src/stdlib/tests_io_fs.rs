use super::{
    resolve_stdlib_constant, resolve_stdlib_function, resolve_stdlib_method,
    resolve_stdlib_runtime_method, resolve_stdlib_value, stdlib_function_param_types,
    stdlib_function_result_count, stdlib_function_result_types, stdlib_function_returns_value,
    StdlibValueInit,
};
use crate::{Program, ValueData, Vm, TYPE_FS_SUB_FS};

#[test]
fn resolves_workspace_filesystem_functions_from_the_registry() {
    let mode_dir = resolve_stdlib_constant("io/fs", "ModeDir").expect("io/fs.ModeDir should exist");
    assert_eq!(mode_dir.typ, "fs.FileMode");
    assert_eq!(mode_dir.value, super::StdlibConstantValue::Int(1 << 31));
    let mode_append =
        resolve_stdlib_constant("io/fs", "ModeAppend").expect("io/fs.ModeAppend should exist");
    assert_eq!(mode_append.typ, "fs.FileMode");
    assert_eq!(mode_append.value, super::StdlibConstantValue::Int(1 << 30));
    let mode_type =
        resolve_stdlib_constant("io/fs", "ModeType").expect("io/fs.ModeType should exist");
    assert_eq!(mode_type.typ, "fs.FileMode");
    assert_eq!(
        mode_type.value,
        super::StdlibConstantValue::Int(
            (1 << 31) | (1 << 27) | (1 << 25) | (1 << 24) | (1 << 26) | (1 << 21) | (1 << 19)
        )
    );
    let mode_perm =
        resolve_stdlib_constant("io/fs", "ModePerm").expect("io/fs.ModePerm should exist");
    assert_eq!(mode_perm.typ, "fs.FileMode");
    assert_eq!(mode_perm.value, super::StdlibConstantValue::Int(0o777));
    let skip_dir = resolve_stdlib_value("io/fs", "SkipDir").expect("io/fs.SkipDir should exist");
    assert_eq!(skip_dir.typ, "error");
    assert_eq!(
        skip_dir.value,
        StdlibValueInit::Constant(super::StdlibConstantValue::Error("skip this directory"))
    );
    let skip_all = resolve_stdlib_value("io/fs", "SkipAll").expect("io/fs.SkipAll should exist");
    assert_eq!(skip_all.typ, "error");
    assert_eq!(
        skip_all.value,
        StdlibValueInit::Constant(super::StdlibConstantValue::Error(
            "skip everything and stop the walk"
        ))
    );

    let dir_fs = resolve_stdlib_function("os", "DirFS").expect("os.DirFS should exist");
    assert!(stdlib_function_returns_value(dir_fs));
    assert_eq!(stdlib_function_result_count(dir_fs), 1);
    assert_eq!(stdlib_function_param_types(dir_fs), Some(&["string"][..]));
    assert_eq!(stdlib_function_result_types(dir_fs), Some(&["fs.FS"][..]));

    let os_read_file = resolve_stdlib_function("os", "ReadFile").expect("os.ReadFile should exist");
    assert!(!stdlib_function_returns_value(os_read_file));
    assert_eq!(stdlib_function_result_count(os_read_file), 2);
    assert_eq!(
        stdlib_function_param_types(os_read_file),
        Some(&["string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(os_read_file),
        Some(&["[]byte", "error"][..])
    );

    let os_write_file =
        resolve_stdlib_function("os", "WriteFile").expect("os.WriteFile should exist");
    assert!(stdlib_function_returns_value(os_write_file));
    assert_eq!(stdlib_function_result_count(os_write_file), 1);
    assert_eq!(
        stdlib_function_param_types(os_write_file),
        Some(&["string", "[]byte", "fs.FileMode"][..])
    );
    assert_eq!(
        stdlib_function_result_types(os_write_file),
        Some(&["error"][..])
    );

    let os_read_dir = resolve_stdlib_function("os", "ReadDir").expect("os.ReadDir should exist");
    assert!(!stdlib_function_returns_value(os_read_dir));
    assert_eq!(stdlib_function_result_count(os_read_dir), 2);
    assert_eq!(
        stdlib_function_param_types(os_read_dir),
        Some(&["string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(os_read_dir),
        Some(&["[]fs.DirEntry", "error"][..])
    );

    let os_stat = resolve_stdlib_function("os", "Stat").expect("os.Stat should exist");
    assert!(!stdlib_function_returns_value(os_stat));
    assert_eq!(stdlib_function_result_count(os_stat), 2);
    assert_eq!(stdlib_function_param_types(os_stat), Some(&["string"][..]));
    assert_eq!(
        stdlib_function_result_types(os_stat),
        Some(&["fs.FileInfo", "error"][..])
    );

    let os_hostname = resolve_stdlib_function("os", "Hostname").expect("os.Hostname should exist");
    assert!(!stdlib_function_returns_value(os_hostname));
    assert_eq!(stdlib_function_result_count(os_hostname), 2);
    assert_eq!(stdlib_function_param_types(os_hostname), Some(&[][..]));
    assert_eq!(
        stdlib_function_result_types(os_hostname),
        Some(&["string", "error"][..])
    );

    let os_executable =
        resolve_stdlib_function("os", "Executable").expect("os.Executable should exist");
    assert!(!stdlib_function_returns_value(os_executable));
    assert_eq!(stdlib_function_result_count(os_executable), 2);
    assert_eq!(stdlib_function_param_types(os_executable), Some(&[][..]));
    assert_eq!(
        stdlib_function_result_types(os_executable),
        Some(&["string", "error"][..])
    );

    let os_getgroups =
        resolve_stdlib_function("os", "Getgroups").expect("os.Getgroups should exist");
    assert!(!stdlib_function_returns_value(os_getgroups));
    assert_eq!(stdlib_function_result_count(os_getgroups), 2);
    assert_eq!(stdlib_function_param_types(os_getgroups), Some(&[][..]));
    assert_eq!(
        stdlib_function_result_types(os_getgroups),
        Some(&["[]int", "error"][..])
    );

    for symbol in [
        "Getuid",
        "Geteuid",
        "Getgid",
        "Getegid",
        "Getpid",
        "Getppid",
        "Getpagesize",
    ] {
        let function = resolve_stdlib_function("os", symbol)
            .unwrap_or_else(|| panic!("os.{symbol} should exist"));
        assert!(stdlib_function_returns_value(function));
        assert_eq!(stdlib_function_result_count(function), 1);
        assert_eq!(stdlib_function_param_types(function), Some(&[][..]));
        assert_eq!(stdlib_function_result_types(function), Some(&["int"][..]));
    }

    let os_is_not_exist =
        resolve_stdlib_function("os", "IsNotExist").expect("os.IsNotExist should exist");
    assert!(stdlib_function_returns_value(os_is_not_exist));
    assert_eq!(stdlib_function_result_count(os_is_not_exist), 1);
    assert_eq!(
        stdlib_function_param_types(os_is_not_exist),
        Some(&["error"][..])
    );
    assert_eq!(
        stdlib_function_result_types(os_is_not_exist),
        Some(&["bool"][..])
    );

    let valid_path =
        resolve_stdlib_function("io/fs", "ValidPath").expect("io/fs.ValidPath should exist");
    assert!(stdlib_function_returns_value(valid_path));
    assert_eq!(stdlib_function_result_count(valid_path), 1);
    assert_eq!(
        stdlib_function_param_types(valid_path),
        Some(&["string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(valid_path),
        Some(&["bool"][..])
    );

    let fs_read_file =
        resolve_stdlib_function("io/fs", "ReadFile").expect("io/fs.ReadFile should exist");
    assert!(!stdlib_function_returns_value(fs_read_file));
    assert_eq!(stdlib_function_result_count(fs_read_file), 2);
    assert_eq!(
        stdlib_function_param_types(fs_read_file),
        Some(&["fs.FS", "string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(fs_read_file),
        Some(&["[]byte", "error"][..])
    );

    let fs_stat = resolve_stdlib_function("io/fs", "Stat").expect("io/fs.Stat should exist");
    assert!(!stdlib_function_returns_value(fs_stat));
    assert_eq!(stdlib_function_result_count(fs_stat), 2);
    assert_eq!(
        stdlib_function_param_types(fs_stat),
        Some(&["fs.FS", "string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(fs_stat),
        Some(&["fs.FileInfo", "error"][..])
    );

    let fs_sub = resolve_stdlib_function("io/fs", "Sub").expect("io/fs.Sub should exist");
    assert!(!stdlib_function_returns_value(fs_sub));
    assert_eq!(stdlib_function_result_count(fs_sub), 2);
    assert_eq!(
        stdlib_function_param_types(fs_sub),
        Some(&["fs.FS", "string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(fs_sub),
        Some(&["fs.FS", "error"][..])
    );

    let fs_glob = resolve_stdlib_function("io/fs", "Glob").expect("io/fs.Glob should exist");
    assert!(!stdlib_function_returns_value(fs_glob));
    assert_eq!(stdlib_function_result_count(fs_glob), 2);
    assert_eq!(
        stdlib_function_param_types(fs_glob),
        Some(&["fs.FS", "string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(fs_glob),
        Some(&["[]string", "error"][..])
    );

    let fs_read_dir =
        resolve_stdlib_function("io/fs", "ReadDir").expect("io/fs.ReadDir should exist");
    assert!(!stdlib_function_returns_value(fs_read_dir));
    assert_eq!(stdlib_function_result_count(fs_read_dir), 2);
    assert_eq!(
        stdlib_function_param_types(fs_read_dir),
        Some(&["fs.FS", "string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(fs_read_dir),
        Some(&["[]fs.DirEntry", "error"][..])
    );

    let fs_walk_dir =
        resolve_stdlib_function("io/fs", "WalkDir").expect("io/fs.WalkDir should exist");
    assert!(stdlib_function_returns_value(fs_walk_dir));
    assert_eq!(stdlib_function_result_count(fs_walk_dir), 1);
    assert_eq!(
        stdlib_function_param_types(fs_walk_dir),
        Some(
            &[
                "fs.FS",
                "string",
                "__gowasm_func__(string, fs.DirEntry, error)->(error)",
            ][..]
        )
    );
    assert_eq!(
        stdlib_function_result_types(fs_walk_dir),
        Some(&["error"][..])
    );

    let fs_file_info_to_dir_entry = resolve_stdlib_function("io/fs", "FileInfoToDirEntry")
        .expect("io/fs.FileInfoToDirEntry should exist");
    assert!(stdlib_function_returns_value(fs_file_info_to_dir_entry));
    assert_eq!(stdlib_function_result_count(fs_file_info_to_dir_entry), 1);
    assert_eq!(
        stdlib_function_param_types(fs_file_info_to_dir_entry),
        Some(&["fs.FileInfo"][..])
    );
    assert_eq!(
        stdlib_function_result_types(fs_file_info_to_dir_entry),
        Some(&["fs.DirEntry"][..])
    );

    let fs_format_dir_entry = resolve_stdlib_function("io/fs", "FormatDirEntry")
        .expect("io/fs.FormatDirEntry should exist");
    assert!(stdlib_function_returns_value(fs_format_dir_entry));
    assert_eq!(stdlib_function_result_count(fs_format_dir_entry), 1);
    assert_eq!(
        stdlib_function_param_types(fs_format_dir_entry),
        Some(&["fs.DirEntry"][..])
    );
    assert_eq!(
        stdlib_function_result_types(fs_format_dir_entry),
        Some(&["string"][..])
    );

    let fs_format_file_info = resolve_stdlib_function("io/fs", "FormatFileInfo")
        .expect("io/fs.FormatFileInfo should exist");
    assert!(stdlib_function_returns_value(fs_format_file_info));
    assert_eq!(stdlib_function_result_count(fs_format_file_info), 1);
    assert_eq!(
        stdlib_function_param_types(fs_format_file_info),
        Some(&["fs.FileInfo"][..])
    );
    assert_eq!(
        stdlib_function_result_types(fs_format_file_info),
        Some(&["string"][..])
    );

    let fs_open = resolve_stdlib_method("fs.FS", "Open").expect("fs.FS.Open should exist");
    assert!(!stdlib_function_returns_value(fs_open));
    assert_eq!(stdlib_function_result_count(fs_open), 2);
    assert_eq!(
        stdlib_function_param_types(fs_open),
        Some(&["fs.FS", "string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(fs_open),
        Some(&["fs.File", "error"][..])
    );

    let fs_close = resolve_stdlib_method("fs.File", "Close").expect("fs.File.Close should exist");
    assert!(stdlib_function_returns_value(fs_close));
    assert_eq!(stdlib_function_result_count(fs_close), 1);
    assert_eq!(
        stdlib_function_param_types(fs_close),
        Some(&["fs.File"][..])
    );
    assert_eq!(stdlib_function_result_types(fs_close), Some(&["error"][..]));

    let fs_stat = resolve_stdlib_method("fs.File", "Stat").expect("fs.File.Stat should exist");
    assert!(!stdlib_function_returns_value(fs_stat));
    assert_eq!(stdlib_function_result_count(fs_stat), 2);
    assert_eq!(stdlib_function_param_types(fs_stat), Some(&["fs.File"][..]));
    assert_eq!(
        stdlib_function_result_types(fs_stat),
        Some(&["fs.FileInfo", "error"][..])
    );

    let fs_read = resolve_stdlib_method("fs.File", "Read").expect("fs.File.Read should exist");
    assert!(!stdlib_function_returns_value(fs_read));
    assert_eq!(stdlib_function_result_count(fs_read), 2);
    assert_eq!(
        stdlib_function_param_types(fs_read),
        Some(&["fs.File", "[]byte"][..])
    );
    assert_eq!(
        stdlib_function_result_types(fs_read),
        Some(&["int", "error"][..])
    );

    let fs_read_dir_file_read_dir = resolve_stdlib_method("fs.ReadDirFile", "ReadDir")
        .expect("fs.ReadDirFile.ReadDir should exist");
    assert!(!stdlib_function_returns_value(fs_read_dir_file_read_dir));
    assert_eq!(stdlib_function_result_count(fs_read_dir_file_read_dir), 2);
    assert_eq!(
        stdlib_function_param_types(fs_read_dir_file_read_dir),
        Some(&["fs.ReadDirFile", "int"][..])
    );
    assert_eq!(
        stdlib_function_result_types(fs_read_dir_file_read_dir),
        Some(&["[]fs.DirEntry", "error"][..])
    );

    let dir_entry_info =
        resolve_stdlib_method("fs.DirEntry", "Info").expect("fs.DirEntry.Info should exist");
    assert!(!stdlib_function_returns_value(dir_entry_info));
    assert_eq!(stdlib_function_result_count(dir_entry_info), 2);
    assert_eq!(
        stdlib_function_param_types(dir_entry_info),
        Some(&["fs.DirEntry"][..])
    );
    assert_eq!(
        stdlib_function_result_types(dir_entry_info),
        Some(&["fs.FileInfo", "error"][..])
    );

    let file_info_name =
        resolve_stdlib_method("fs.FileInfo", "Name").expect("fs.FileInfo.Name should exist");
    assert!(stdlib_function_returns_value(file_info_name));
    assert_eq!(stdlib_function_result_count(file_info_name), 1);
    assert_eq!(
        stdlib_function_param_types(file_info_name),
        Some(&["fs.FileInfo"][..])
    );
    assert_eq!(
        stdlib_function_result_types(file_info_name),
        Some(&["string"][..])
    );

    let file_info_is_dir =
        resolve_stdlib_method("fs.FileInfo", "IsDir").expect("fs.FileInfo.IsDir should exist");
    assert!(stdlib_function_returns_value(file_info_is_dir));
    assert_eq!(stdlib_function_result_count(file_info_is_dir), 1);
    assert_eq!(
        stdlib_function_param_types(file_info_is_dir),
        Some(&["fs.FileInfo"][..])
    );
    assert_eq!(
        stdlib_function_result_types(file_info_is_dir),
        Some(&["bool"][..])
    );

    let file_info_size =
        resolve_stdlib_method("fs.FileInfo", "Size").expect("fs.FileInfo.Size should exist");
    assert!(stdlib_function_returns_value(file_info_size));
    assert_eq!(stdlib_function_result_count(file_info_size), 1);
    assert_eq!(
        stdlib_function_param_types(file_info_size),
        Some(&["fs.FileInfo"][..])
    );
    assert_eq!(
        stdlib_function_result_types(file_info_size),
        Some(&["int"][..])
    );

    let file_info_mode =
        resolve_stdlib_method("fs.FileInfo", "Mode").expect("fs.FileInfo.Mode should exist");
    assert!(stdlib_function_returns_value(file_info_mode));
    assert_eq!(stdlib_function_result_count(file_info_mode), 1);
    assert_eq!(
        stdlib_function_param_types(file_info_mode),
        Some(&["fs.FileInfo"][..])
    );
    assert_eq!(
        stdlib_function_result_types(file_info_mode),
        Some(&["fs.FileMode"][..])
    );

    let file_info_mod_time =
        resolve_stdlib_method("fs.FileInfo", "ModTime").expect("fs.FileInfo.ModTime should exist");
    assert!(stdlib_function_returns_value(file_info_mod_time));
    assert_eq!(stdlib_function_result_count(file_info_mod_time), 1);
    assert_eq!(
        stdlib_function_param_types(file_info_mod_time),
        Some(&["fs.FileInfo"][..])
    );
    assert_eq!(
        stdlib_function_result_types(file_info_mod_time),
        Some(&["time.Time"][..])
    );

    let file_info_sys =
        resolve_stdlib_method("fs.FileInfo", "Sys").expect("fs.FileInfo.Sys should exist");
    assert!(stdlib_function_returns_value(file_info_sys));
    assert_eq!(stdlib_function_result_count(file_info_sys), 1);
    assert_eq!(
        stdlib_function_param_types(file_info_sys),
        Some(&["fs.FileInfo"][..])
    );
    assert_eq!(
        stdlib_function_result_types(file_info_sys),
        Some(&["interface{}"][..])
    );

    let file_mode_is_dir =
        resolve_stdlib_method("fs.FileMode", "IsDir").expect("fs.FileMode.IsDir should exist");
    assert!(stdlib_function_returns_value(file_mode_is_dir));
    assert_eq!(stdlib_function_result_count(file_mode_is_dir), 1);
    assert_eq!(
        stdlib_function_param_types(file_mode_is_dir),
        Some(&["fs.FileMode"][..])
    );
    assert_eq!(
        stdlib_function_result_types(file_mode_is_dir),
        Some(&["bool"][..])
    );

    let file_mode_is_regular = resolve_stdlib_method("fs.FileMode", "IsRegular")
        .expect("fs.FileMode.IsRegular should exist");
    assert!(stdlib_function_returns_value(file_mode_is_regular));
    assert_eq!(stdlib_function_result_count(file_mode_is_regular), 1);
    assert_eq!(
        stdlib_function_param_types(file_mode_is_regular),
        Some(&["fs.FileMode"][..])
    );
    assert_eq!(
        stdlib_function_result_types(file_mode_is_regular),
        Some(&["bool"][..])
    );

    let file_mode_type =
        resolve_stdlib_method("fs.FileMode", "Type").expect("fs.FileMode.Type should exist");
    assert!(stdlib_function_returns_value(file_mode_type));
    assert_eq!(stdlib_function_result_count(file_mode_type), 1);
    assert_eq!(
        stdlib_function_param_types(file_mode_type),
        Some(&["fs.FileMode"][..])
    );
    assert_eq!(
        stdlib_function_result_types(file_mode_type),
        Some(&["fs.FileMode"][..])
    );

    let file_mode_string =
        resolve_stdlib_method("fs.FileMode", "String").expect("fs.FileMode.String should exist");
    assert!(stdlib_function_returns_value(file_mode_string));
    assert_eq!(stdlib_function_result_count(file_mode_string), 1);
    assert_eq!(
        stdlib_function_param_types(file_mode_string),
        Some(&["fs.FileMode"][..])
    );
    assert_eq!(
        stdlib_function_result_types(file_mode_string),
        Some(&["string"][..])
    );

    let file_mode_perm =
        resolve_stdlib_method("fs.FileMode", "Perm").expect("fs.FileMode.Perm should exist");
    assert!(stdlib_function_returns_value(file_mode_perm));
    assert_eq!(stdlib_function_result_count(file_mode_perm), 1);
    assert_eq!(
        stdlib_function_param_types(file_mode_perm),
        Some(&["fs.FileMode"][..])
    );
    assert_eq!(
        stdlib_function_result_types(file_mode_perm),
        Some(&["fs.FileMode"][..])
    );

    let dir_entry_name =
        resolve_stdlib_method("fs.DirEntry", "Name").expect("fs.DirEntry.Name should exist");
    assert!(stdlib_function_returns_value(dir_entry_name));
    assert_eq!(stdlib_function_result_count(dir_entry_name), 1);
    assert_eq!(
        stdlib_function_param_types(dir_entry_name),
        Some(&["fs.DirEntry"][..])
    );
    assert_eq!(
        stdlib_function_result_types(dir_entry_name),
        Some(&["string"][..])
    );

    let dir_entry_is_dir =
        resolve_stdlib_method("fs.DirEntry", "IsDir").expect("fs.DirEntry.IsDir should exist");
    assert!(stdlib_function_returns_value(dir_entry_is_dir));
    assert_eq!(stdlib_function_result_count(dir_entry_is_dir), 1);
    assert_eq!(
        stdlib_function_param_types(dir_entry_is_dir),
        Some(&["fs.DirEntry"][..])
    );
    assert_eq!(
        stdlib_function_result_types(dir_entry_is_dir),
        Some(&["bool"][..])
    );

    let dir_entry_type =
        resolve_stdlib_method("fs.DirEntry", "Type").expect("fs.DirEntry.Type should exist");
    assert!(stdlib_function_returns_value(dir_entry_type));
    assert_eq!(stdlib_function_result_count(dir_entry_type), 1);
    assert_eq!(
        stdlib_function_param_types(dir_entry_type),
        Some(&["fs.DirEntry"][..])
    );
    assert_eq!(
        stdlib_function_result_types(dir_entry_type),
        Some(&["fs.FileMode"][..])
    );
}

#[test]
fn subfs_wrapper_resolves_fs_open_runtime_methods() {
    let fs_open = resolve_stdlib_method("fs.FS", "Open").expect("fs.FS.Open should exist");
    assert_eq!(
        resolve_stdlib_runtime_method(TYPE_FS_SUB_FS, "Open"),
        Some(fs_open)
    );
}

#[test]
fn workspace_handles_preserve_os_origin_in_stat_and_dir_entry_info() {
    let program = Program {
        functions: Vec::new(),
        methods: Vec::new(),
        global_count: 0,
        entry_function: 0,
    };
    let mut vm = Vm::new();
    vm.workspace_files.insert("assets/a.txt".into(), "a".into());
    vm.workspace_files
        .insert("assets/nested/b.txt".into(), "b".into());

    let generic_file =
        super::workspace_fs_impl::open_workspace_file(&mut vm, None, "assets/a.txt", false)
            .expect("generic workspace file should open");
    let generic_stat =
        super::workspace_fs_impl::stat_workspace_file(&vm, &program, "fs.File.Stat", &generic_file)
            .expect("generic stat should succeed");
    assert_eq!(
        super::workspace_fs_impl::same_file_workspace_path(&generic_stat[0]),
        None
    );

    let os_file =
        super::workspace_fs_impl::open_workspace_file(&mut vm, None, "assets/a.txt", true)
            .expect("os workspace file should open");
    let os_stat =
        super::workspace_fs_impl::stat_workspace_file(&vm, &program, "fs.File.Stat", &os_file)
            .expect("os stat should succeed");
    assert_eq!(
        super::workspace_fs_impl::same_file_workspace_path(&os_stat[0]),
        Some("assets/a.txt".into())
    );

    let dir = super::workspace_fs_impl::open_workspace_file(&mut vm, None, "assets", true)
        .expect("workspace directory should open");
    let dir_results = super::workspace_fs_impl::read_workspace_file_dir_entries(
        &mut vm,
        &program,
        "os.ReadDir",
        &dir,
        -1,
    )
    .expect("workspace dir read should succeed");
    let ValueData::Slice(entries) = &dir_results[0].data else {
        panic!("expected directory entries slice");
    };
    let visible_entries = entries.values_snapshot();
    let entry = visible_entries
        .iter()
        .find(|entry| {
            super::workspace_fs_impl::dir_entry_name(&mut vm, &program, "fs.DirEntry.Name", entry)
                .map(|name| name == "a.txt")
                .unwrap_or(false)
        })
        .cloned()
        .expect("a.txt directory entry should exist");
    let entry_info =
        super::workspace_fs_impl::dir_entry_info(&mut vm, &program, "fs.DirEntry.Info", &entry)
            .expect("dir entry info should succeed");
    assert_eq!(
        super::workspace_fs_impl::same_file_workspace_path(&entry_info[0]),
        Some("assets/a.txt".into())
    );
}

#[test]
fn workspace_handles_report_closed_read_stat_and_readdir_errors() {
    let program = Program {
        functions: Vec::new(),
        methods: Vec::new(),
        global_count: 0,
        entry_function: 0,
    };
    let mut vm = Vm::new();
    vm.workspace_files.insert("data.txt".into(), "alpha".into());
    vm.workspace_files
        .insert("nested/child.txt".into(), "child".into());

    let file = super::workspace_fs_impl::open_workspace_file(&mut vm, None, "data.txt", false)
        .expect("workspace file should open");
    let buffer = crate::Value::slice(vec![crate::Value::int(0), crate::Value::int(0)]);
    let read_results =
        super::io_fs_impl::io_fs_file_read(&mut vm, &program, &[file.clone(), buffer])
            .expect("workspace file read should succeed");
    let ValueData::Int(read_count) = read_results[0].data else {
        panic!("expected int read count");
    };
    assert_eq!(read_count, 2);
    assert!(matches!(read_results[1].data, ValueData::Nil));

    let close_result =
        super::workspace_fs_impl::close_workspace_file(&mut vm, &program, "fs.File.Close", &file)
            .expect("workspace file close should succeed");
    assert!(matches!(close_result.data, ValueData::Nil));

    let closed_read = super::io_fs_impl::io_fs_file_read(
        &mut vm,
        &program,
        &[file.clone(), crate::Value::slice(vec![])],
    )
    .expect("closed file read should return an error result");
    let ValueData::Int(closed_read_count) = closed_read[0].data else {
        panic!("expected int closed-read count");
    };
    assert_eq!(closed_read_count, 0);
    let ValueData::Error(closed_read_error) = &closed_read[1].data else {
        panic!("expected closed-read error");
    };
    assert_eq!(
        closed_read_error.message,
        "read data.txt: file already closed"
    );

    let closed_stat =
        super::workspace_fs_impl::stat_workspace_file(&vm, &program, "fs.File.Stat", &file)
            .expect("closed file stat should return an error result");
    assert!(matches!(closed_stat[0].data, ValueData::Nil));
    let ValueData::Error(closed_stat_error) = &closed_stat[1].data else {
        panic!("expected closed-stat error");
    };
    assert_eq!(
        closed_stat_error.message,
        "stat data.txt: file already closed"
    );

    let dir = super::workspace_fs_impl::open_workspace_file(&mut vm, None, "nested", false)
        .expect("workspace directory should open");
    let dir_close =
        super::workspace_fs_impl::close_workspace_file(&mut vm, &program, "fs.File.Close", &dir)
            .expect("workspace directory close should succeed");
    assert!(matches!(dir_close.data, ValueData::Nil));
    let closed_readdir = super::workspace_fs_impl::read_workspace_file_dir_entries(
        &mut vm,
        &program,
        "fs.ReadDirFile.ReadDir",
        &dir,
        -1,
    )
    .expect("closed directory readdir should return an error result");
    let ValueData::Slice(closed_readdir_entries) = &closed_readdir[0].data else {
        panic!("expected typed nil dir-entry slice");
    };
    assert!(closed_readdir_entries.is_nil);
    let ValueData::Error(closed_readdir_error) = &closed_readdir[1].data else {
        panic!("expected closed-readdir error");
    };
    assert_eq!(
        closed_readdir_error.message,
        "readdir nested: file already closed"
    );
}
