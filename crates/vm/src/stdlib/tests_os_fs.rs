use super::{
    resolve_stdlib_function, stdlib_function_param_types, stdlib_function_result_types,
    stdlib_function_returns_value,
};
use crate::{Program, Vm};

fn empty_program() -> Program {
    Program {
        functions: Vec::new(),
        methods: Vec::new(),
        global_count: 0,
        entry_function: 0,
    }
}

fn labels(vm: &mut Vm, program: &Program, entries: &[crate::Value]) -> Vec<String> {
    entries
        .iter()
        .map(|entry| {
            let mut name =
                super::workspace_fs_impl::dir_entry_name(vm, program, "fs.DirEntry.Name", entry)
                    .expect("dir entry should have a name");
            if super::workspace_fs_impl::dir_entry_is_dir(vm, program, "fs.DirEntry.IsDir", entry)
                .expect("dir entry should report IsDir")
            {
                name.push('/');
            }
            name
        })
        .collect()
}

#[test]
fn resolves_os_directory_mutation_functions_from_the_registry() {
    let mkdir_all = resolve_stdlib_function("os", "MkdirAll").expect("os.MkdirAll should exist");
    assert!(stdlib_function_returns_value(mkdir_all));
    assert_eq!(
        stdlib_function_param_types(mkdir_all),
        Some(&["string", "fs.FileMode"][..])
    );
    assert_eq!(
        stdlib_function_result_types(mkdir_all),
        Some(&["error"][..])
    );

    let remove_all = resolve_stdlib_function("os", "RemoveAll").expect("os.RemoveAll should exist");
    assert!(stdlib_function_returns_value(remove_all));
    assert_eq!(
        stdlib_function_param_types(remove_all),
        Some(&["string"][..])
    );
    assert_eq!(
        stdlib_function_result_types(remove_all),
        Some(&["error"][..])
    );
}

#[test]
fn workspace_directory_mutations_preserve_empty_dirs_and_recursive_removals() {
    let program = empty_program();
    let mut vm = Vm::new();
    vm.workspace_files.insert("keep.txt".into(), "keep".into());
    vm.workspace_files
        .insert("existing/file.txt".into(), "file".into());
    vm.workspace_files
        .insert("existing/nested/child.txt".into(), "child".into());

    super::workspace_fs_impl::mkdir_all_os_workspace_path(&mut vm, "empty/deep")
        .expect("mkdir all should succeed");
    let root_entries = super::workspace_fs_impl::read_workspace_dir_entries(&vm, None, ".", true)
        .expect("root entries should resolve");
    assert_eq!(
        labels(&mut vm, &program, &root_entries),
        vec!["empty/", "existing/", "keep.txt"]
    );

    let empty_entries =
        super::workspace_fs_impl::read_workspace_dir_entries(&vm, None, "empty", true)
            .expect("empty dir should resolve");
    assert_eq!(labels(&mut vm, &program, &empty_entries), vec!["deep/"]);

    let empty_deep_entries =
        super::workspace_fs_impl::read_workspace_dir_entries(&vm, None, "empty/deep", true)
            .expect("deep dir should resolve");
    assert!(empty_deep_entries.is_empty());

    super::workspace_fs_impl::remove_all_os_workspace_path(&mut vm, "existing")
        .expect("remove all should succeed");
    let root_after = super::workspace_fs_impl::read_workspace_dir_entries(&vm, None, ".", true)
        .expect("root entries should still resolve");
    assert_eq!(
        labels(&mut vm, &program, &root_after),
        vec!["empty/", "keep.txt"]
    );
    assert!(
        super::workspace_fs_impl::open_workspace_file(&mut vm, None, "existing", true).is_err(),
        "removed directory should no longer open"
    );
}

#[test]
fn workspace_file_writes_update_the_mutable_workspace_tree() {
    let mut vm = Vm::new();
    vm.workspace_files
        .insert("assets/existing.txt".into(), "existing".into());

    super::workspace_fs_impl::write_os_workspace_file(&mut vm, "/assets/new.txt", b"alpha")
        .expect("write should succeed");
    super::workspace_fs_impl::write_os_workspace_file(&mut vm, "assets/new.txt", b"beta")
        .expect("rewrite should succeed");

    assert_eq!(
        super::workspace_fs_impl::read_os_workspace_file(&vm, "assets/new.txt")
            .expect("written file should be readable"),
        b"beta",
    );
    assert!(
        super::workspace_fs_impl::write_os_workspace_file(&mut vm, "missing/new.txt", b"x")
            .is_err(),
        "missing parents should fail"
    );
    assert!(
        super::workspace_fs_impl::write_os_workspace_file(&mut vm, "assets", b"x").is_err(),
        "directory writes should fail"
    );
}
