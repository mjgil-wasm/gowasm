use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn os_exit_terminates_program() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    fmt.Println("before")
    os.Exit(0)
    fmt.Println("unreachable")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let _ = vm.run_program(&program);
    assert_eq!(vm.stdout(), "before\n");
}

#[test]
fn os_getenv_returns_empty_for_missing() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    fmt.Printf("[%s]", os.Getenv("MISSING_KEY"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[]");
}

#[test]
fn os_setenv_and_getenv_round_trip() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    os.Setenv("MY_KEY", "my_value")
    fmt.Println(os.Getenv("MY_KEY"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "my_value\n");
}

#[test]
fn os_setenv_overwrites_previous() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    os.Setenv("KEY", "first")
    os.Setenv("KEY", "second")
    fmt.Println(os.Getenv("KEY"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "second\n");
}

#[test]
fn os_getenv_with_preloaded_env() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    fmt.Println(os.Getenv("PRELOADED"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.env.insert("PRELOADED".to_string(), "hello".to_string());
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello\n");
}

#[test]
fn os_unsetenv_removes_key() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    os.Setenv("KEY", "value")
    fmt.Println(os.Getenv("KEY"))
    os.Unsetenv("KEY")
    fmt.Printf("[%s]", os.Getenv("KEY"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "value\n[]");
}

#[test]
fn os_clearenv_removes_all() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    os.Setenv("A", "1")
    os.Setenv("B", "2")
    os.Clearenv()
    fmt.Printf("[%s][%s]", os.Getenv("A"), os.Getenv("B"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[][]");
}

#[test]
fn os_lookup_env_found() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    os.Setenv("KEY", "hello")
    val, ok := os.LookupEnv("KEY")
    fmt.Println(val, ok)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "hello true\n");
}

#[test]
fn os_lookup_env_missing() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    val, ok := os.LookupEnv("NOPE")
    fmt.Println(val, ok)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), " false\n");
}

#[test]
fn os_environ_returns_sorted_key_value_pairs() {
    let source = r#"
package main
import "fmt"
import "os"
import "strings"

func main() {
    os.Setenv("B", "2")
    os.Setenv("A", "1")
    fmt.Println(strings.Join(os.Environ(), ","))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "A=1,B=2\n");
}

#[test]
fn os_expand_env_uses_vm_environment_and_matches_go_syntax_rules() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    os.Setenv("NAME", "world")
    fmt.Printf("[%s]\n", os.ExpandEnv("hello $NAME ${MISSING} $"))
    fmt.Printf("[%s]\n", os.ExpandEnv("bad ${} ${"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[hello world  $]\n[bad  ]\n");
}

#[test]
fn os_expand_uses_callback_mapping_and_captures() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    prefix := "<"
    fmt.Printf("[%s]\n", os.Expand("hello $A ${MISSING} $", func(name string) string {
        if name == "A" {
            return "1"
        }
        return prefix + name + ">"
    }))
    fmt.Printf("[%s]\n", os.Expand("bad ${} ${", func(name string) string {
        return "x"
    }))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "[hello 1 <MISSING> $]\n[bad  ]\n");
}

#[test]
fn os_hostname_and_executable_follow_js_wasm_behavior() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    host, hostErr := os.Hostname()
    exe, exeErr := os.Executable()
    fmt.Println(host, hostErr == nil)
    fmt.Println(exe == "", exeErr != nil)
    fmt.Println(exeErr.Error())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "js true\ntrue true\nExecutable not implemented for js\n"
    );
}

#[test]
fn os_identity_helpers_return_browser_safe_sentinels() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    fmt.Println(
        os.Getuid(),
        os.Geteuid(),
        os.Getgid(),
        os.Getegid(),
        os.Getpid(),
        os.Getppid(),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "-1 -1 -1 -1 -1 -1\n");
}

#[test]
fn os_getpagesize_matches_wasm_runtime_page_size() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    fmt.Println(os.Getpagesize())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "65536\n");
}

#[test]
fn os_getgroups_returns_browser_not_implemented_error() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    groups, err := os.Getgroups()
    fmt.Printf("%d %t\n", len(groups), err != nil)
    fmt.Println(err.Error())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 true\nGetgroups not implemented for js\n");
}

#[test]
fn os_read_dir_lists_workspace_directory_entries() {
    let source = r#"
package main
import "fmt"
import "os"
import "strings"

func main() {
    root, rootErr := os.ReadDir(".")
    entries, err := os.ReadDir("assets")
    nested, nestedErr := os.ReadDir("assets/nested")
    fileEntries, fileErr := os.ReadDir("assets/a.txt")
    missing, missingErr := os.ReadDir("missing")

    var topNames []string
    for _, entry := range root {
        name := entry.Name()
        if entry.IsDir() {
            name += "/"
        }
        topNames = append(topNames, name)
    }

    var rootNames []string
    for _, entry := range entries {
        name := entry.Name()
        if entry.IsDir() {
            name += "/"
        }
        rootNames = append(rootNames, name)
    }

    var nestedNames []string
    for _, entry := range nested {
        name := entry.Name()
        if entry.IsDir() {
            name += "/"
        }
        nestedNames = append(nestedNames, name)
    }

    fmt.Println(strings.Join(topNames, ","), rootErr == nil)
    fmt.Println(strings.Join(rootNames, ","), err == nil)
    fmt.Println(strings.Join(nestedNames, ","), nestedErr == nil)
    fmt.Println(fileEntries == nil, fileErr != nil)
    fmt.Println(missing == nil, missingErr != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files.insert("root.txt".into(), "root".into());
    vm.workspace_files.insert("assets/a.txt".into(), "a".into());
    vm.workspace_files
        .insert("assets/nested/b.txt".into(), "b".into());
    vm.workspace_files
        .insert("assets/other/c.txt".into(), "c".into());
    vm.workspace_files.insert("assets/z.txt".into(), "z".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "assets/,root.txt true\na.txt,nested/,other/,z.txt true\nb.txt true\ntrue true\ntrue true\n"
    );
}

#[test]
fn os_stat_reports_workspace_file_and_directory_info() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    fileInfo, fileErr := os.Stat("assets/a.txt")
    dirInfo, dirErr := os.Stat("assets/nested")
    missing, missingErr := os.Stat("missing")

    fmt.Println(fileErr == nil, fileInfo.Name(), fileInfo.IsDir())
    fmt.Println(dirErr == nil, dirInfo.Name(), dirInfo.IsDir())
    fmt.Println(missing == nil, missingErr != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files.insert("assets/a.txt".into(), "a".into());
    vm.workspace_files
        .insert("assets/nested/b.txt".into(), "b".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true a.txt false\ntrue nested true\ntrue true\n"
    );
}

#[test]
fn os_lstat_reports_workspace_file_and_directory_info() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    fileInfo, fileErr := os.Lstat("assets/a.txt")
    dirInfo, dirErr := os.Lstat("assets/nested")
    missing, missingErr := os.Lstat("missing")

    fmt.Println(fileErr == nil, fileInfo.Name(), fileInfo.IsDir())
    fmt.Println(dirErr == nil, dirInfo.Name(), dirInfo.IsDir())
    fmt.Println(missing == nil, missingErr != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files.insert("assets/a.txt".into(), "a".into());
    vm.workspace_files
        .insert("assets/nested/b.txt".into(), "b".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true a.txt false\ntrue nested true\ntrue true\n"
    );
}

#[test]
fn os_getwd_reports_workspace_root() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    dir, err := os.Getwd()
    fmt.Println(dir, err == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "/ true\n");
}

#[test]
fn os_directory_helpers_follow_unix_env_model() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    fmt.Println(os.TempDir())
    os.Setenv("TMPDIR", "/sandbox/tmp")
    fmt.Println(os.TempDir())

    os.Clearenv()
    home, homeErr := os.UserHomeDir()
    fmt.Println(home == "", homeErr.Error())

    os.Setenv("HOME", "/users/alice")
    home, homeErr = os.UserHomeDir()
    cache, cacheErr := os.UserCacheDir()
    config, configErr := os.UserConfigDir()
    fmt.Println(home, homeErr == nil)
    fmt.Println(cache, cacheErr == nil)
    fmt.Println(config, configErr == nil)

    os.Unsetenv("HOME")
    os.Setenv("XDG_CACHE_HOME", "/cache")
    os.Setenv("XDG_CONFIG_HOME", "/config")
    cache, cacheErr = os.UserCacheDir()
    config, configErr = os.UserConfigDir()
    fmt.Println(cache, cacheErr == nil)
    fmt.Println(config, configErr == nil)

    os.Clearenv()
    _, cacheErr = os.UserCacheDir()
    _, configErr = os.UserConfigDir()
    fmt.Println(cacheErr.Error())
    fmt.Println(configErr.Error())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "/tmp\n/sandbox/tmp\ntrue $HOME is not defined\n/users/alice true\n/users/alice/.cache true\n/users/alice/.config true\n/cache true\n/config true\nneither $XDG_CACHE_HOME nor $HOME are defined\nneither $XDG_CONFIG_HOME nor $HOME are defined\n"
    );
}

#[test]
fn os_is_path_separator_matches_workspace_slashes() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    fmt.Println(os.IsPathSeparator('/'), os.IsPathSeparator('a'), os.IsPathSeparator('\\'))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true false false\n");
}

#[test]
fn os_path_separator_constant_matches_workspace_model() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    fmt.Println(os.PathSeparator == '/', os.IsPathSeparator(os.PathSeparator))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true\n");
}

#[test]
fn os_path_list_separator_and_dev_null_constants_match_workspace_model() {
    let source = r#"
package main
import "fmt"
import "os"

func main() {
    fmt.Println(os.PathListSeparator == ':', os.DevNull == "/dev/null")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true\n");
}

#[test]
fn os_is_not_exist_matches_workspace_missing_errors() {
    let source = r#"
package main
import "errors"
import "fmt"
import "os"

func main() {
    _, readErr := os.ReadFile("missing")
    _, dirErr := os.ReadDir("missing")
    _, statErr := os.Stat("missing")
    _, invalidErr := os.ReadFile("")
    other := errors.New("other")

    fmt.Println(os.IsNotExist(readErr), os.IsNotExist(dirErr), os.IsNotExist(statErr))
    fmt.Println(os.IsNotExist(invalidErr), os.IsNotExist(other), os.IsNotExist(nil))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true true true\nfalse false false\n");
}

#[test]
fn os_same_file_preserves_dirfs_and_readdir_workspace_origin() {
    let source = r#"
package main
import "fmt"
import "io/fs"
import "os"

func main() {
    fileInfo, _ := os.Stat("assets/a.txt")
    samePath, _ := os.Lstat("assets/a.txt")
    sameShape, _ := os.Stat("other/a.txt")
    dirInfo, _ := os.Stat("assets")

    fsys := os.DirFS(".")
    file, _ := fsys.Open("assets/a.txt")
    openedInfo, _ := file.Stat()
    entries, _ := os.ReadDir("assets")
    entryInfo, _ := entries[0].Info()
    fsStatInfo, _ := fs.Stat(fsys, "assets/a.txt")
    fsEntries, _ := fs.ReadDir(fsys, "assets")
    fsEntryInfo, _ := fsEntries[0].Info()

    fmt.Println(os.SameFile(fileInfo, samePath))
    fmt.Println(os.SameFile(fileInfo, sameShape))
    fmt.Println(os.SameFile(fileInfo, dirInfo))
    fmt.Println(os.SameFile(fileInfo, openedInfo))
    fmt.Println(os.SameFile(fileInfo, entryInfo))
    fmt.Println(os.SameFile(fileInfo, fsStatInfo))
    fmt.Println(os.SameFile(fileInfo, fsEntryInfo))
    fmt.Println(os.SameFile(nil, fileInfo))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files.insert("assets/a.txt".into(), "x".into());
    vm.workspace_files
        .insert("assets/nested/b.txt".into(), "b".into());
    vm.workspace_files.insert("other/a.txt".into(), "x".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true\nfalse\nfalse\ntrue\ntrue\ntrue\ntrue\nfalse\n"
    );
}

#[test]
fn os_error_values_and_helpers_match_wrapped_os_errors() {
    let source = r#"
package main
import "errors"
import "fmt"
import "os"

var globalErr = os.ErrClosed
var invalid = os.ErrInvalid
var timeout = os.ErrDeadlineExceeded

func main() {
    _, missingErr := os.ReadFile("missing")
    wrappedMissing := fmt.Errorf("wrap: %w", missingErr)
    wrappedPerm := fmt.Errorf("wrap: %w", os.ErrPermission)
    wrappedExist := fmt.Errorf("wrap: %w", os.ErrExist)
    wrappedTimeout := fmt.Errorf("wrap: %w", os.ErrDeadlineExceeded)
    syscallErr := os.NewSyscallError("open", os.ErrNotExist)
    nilSyscallErr := os.NewSyscallError("open", nil)
    timeoutSyscallErr := os.NewSyscallError("read", os.ErrDeadlineExceeded)

    fmt.Println(os.ErrInvalid.Error())
    fmt.Println(os.ErrPermission.Error())
    fmt.Println(os.ErrExist.Error())
    fmt.Println(os.ErrNotExist.Error())
    fmt.Println(os.ErrClosed.Error())
    fmt.Println(os.ErrDeadlineExceeded.Error())
    fmt.Println(os.ErrNoDeadline.Error())
    fmt.Println(os.IsExist(os.ErrExist), os.IsExist(wrappedExist), os.IsExist(missingErr))
    fmt.Println(os.IsNotExist(os.ErrNotExist), os.IsNotExist(wrappedMissing))
    fmt.Println(os.IsPermission(os.ErrPermission), os.IsPermission(wrappedPerm), os.IsPermission(missingErr))
    fmt.Println(os.IsTimeout(os.ErrDeadlineExceeded), os.IsTimeout(wrappedTimeout), os.IsTimeout(os.ErrInvalid), os.IsTimeout(missingErr), os.IsTimeout(nil))
    fmt.Println(syscallErr.Error())
    fmt.Println(nilSyscallErr == nil)
    fmt.Println(errors.Is(syscallErr, os.ErrNotExist), errors.Is(timeoutSyscallErr, os.ErrDeadlineExceeded))
    fmt.Println(os.IsTimeout(timeoutSyscallErr), os.IsTimeout(os.ErrNoDeadline))
    fmt.Println(globalErr == os.ErrClosed)
    fmt.Println(invalid == os.ErrInvalid, timeout == os.ErrDeadlineExceeded)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "invalid argument\npermission denied\nfile already exists\nfile does not exist\nfile already closed\ni/o timeout\nfile type does not support deadline\ntrue true false\ntrue true\ntrue true false\ntrue true false false false\nopen: file does not exist\ntrue\ntrue true\ntrue false\ntrue\ntrue true\n"
    );
}

#[test]
fn os_expand_rejects_wrong_callback_signature() {
    let source = r#"
package main
import "os"

func main() {
    _ = os.Expand("$A", func(name string) bool {
        return name == "A"
    })
}
"#;

    let error = compile_source(source).expect_err("program should not compile");
    assert!(
        error.to_string().contains(
            "function value of type `func(string) bool` is not assignable to `func(string) string` in the current subset"
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn os_raw_host_file_apis_stay_outside_the_supported_subset() {
    let cases = [
        (
            "Open",
            r#"
package main
import "os"

func main() {
    _, _ = os.Open("file.txt")
}
"#,
        ),
        (
            "Create",
            r#"
package main
import "os"

func main() {
    _, _ = os.Create("file.txt")
}
"#,
        ),
    ];

    for (selector, source) in cases {
        let error = compile_source(source).expect_err("program should not compile");
        assert!(
            error.to_string().contains(&format!(
                "package selector `os.{selector}` is not supported in the current subset"
            )),
            "unexpected error for os.{selector}: {error}"
        );
    }
}
