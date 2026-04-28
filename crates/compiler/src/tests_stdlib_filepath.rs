use super::{compile_source, compile_workspace, SourceInput};
use gowasm_vm::Vm;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[test]
fn compiles_and_runs_filepath_helpers() {
    let source = r#"
package main
import "fmt"
import "path/filepath"

func main() {
    fmt.Println(
        filepath.Base("/a/b/"),
        filepath.Clean("a/../../b"),
        filepath.Dir("/a/b/"),
        filepath.Ext("archive.tar.gz"),
        filepath.IsAbs("/a/b/"),
        filepath.IsAbs("a/../../b"),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "b ../b /a/b .gz true false\n");
}

#[test]
fn compiles_and_runs_filepath_split_and_join() {
    let source = r#"
package main
import "fmt"
import "path/filepath"

func main() {
    dir, file := filepath.Split("a/b")
    parentDir, parentFile := filepath.Split("../a")
    fmt.Println(dir, file, parentDir, parentFile)
    fmt.Println(
        "[" + filepath.Join() + "]",
        filepath.Join("", "a", "b"),
        filepath.Join("/a", "../b", "c"),
    )
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "a/ b ../ a\n[] a/b /b/c\n");
}

#[test]
fn compiles_and_runs_filepath_match() {
    let source = r#"
package main
import "fmt"
import "path/filepath"

func main() {
    matched, err := filepath.Match("a?c", "abc")
    slashMiss, slashErr := filepath.Match("*", "a/b")
    bad, badErr := filepath.Match("[", "go")
    fmt.Println(matched, err, slashMiss, slashErr, bad, badErr)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true <nil> false <nil> false syntax error in pattern\n"
    );
}

#[test]
fn compiles_and_runs_filepath_function_values() {
    let source = r#"
package main
import "fmt"
import "path/filepath"

func main() {
    clean := filepath.Clean
    fmt.Println(clean("a/../b"))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "b\n");
}

#[test]
fn compiles_and_runs_filepath_slash_helpers() {
    let source = r#"
package main
import "fmt"
import "path/filepath"
import "strings"

func main() {
    fmt.Println(filepath.ToSlash("a/b/c"))
    fmt.Println(filepath.FromSlash("a/b/c"))
    fmt.Println(strings.Join(filepath.SplitList("a:b::c"), ","))
    fmt.Println(len(filepath.SplitList("")))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "a/b/c\na/b/c\na,b,,c\n0\n");
}

#[test]
fn compiles_and_runs_filepath_rel_and_volume_name() {
    let source = r#"
package main
import "fmt"
import "path/filepath"

func main() {
    same, sameErr := filepath.Rel("a/b", "a/b")
    down, downErr := filepath.Rel("/a/b", "/a/c/d")
    up, upErr := filepath.Rel("a/b/c", "a/d")
    bad, badErr := filepath.Rel("../a", "b")
    fmt.Printf("[%s]\n", filepath.VolumeName("/a/b"))
    fmt.Println(same, sameErr)
    fmt.Println(down, downErr)
    fmt.Println(up, upErr)
    fmt.Println(bad == "", badErr.Error())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "[]\n. <nil>\n../c/d <nil>\n../../d <nil>\ntrue Rel: can't make b relative to ../a\n"
    );
}

#[test]
fn compiles_and_runs_filepath_is_local_and_localize() {
    let source = r#"
package main
import "fmt"
import "path/filepath"

func main() {
    fmt.Println(
        filepath.IsLocal("a/b"),
        filepath.IsLocal("./a"),
        filepath.IsLocal("."),
        filepath.IsLocal("../a"),
        filepath.IsLocal("/a"),
        filepath.IsLocal(""),
    )
    local, localErr := filepath.Localize("a/b")
    dot, dotErr := filepath.Localize(".")
    bad, badErr := filepath.Localize("../a")
    fmt.Println(local, localErr)
    fmt.Println(dot, dotErr)
    fmt.Println(bad == "", badErr.Error())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true true true false false false\na/b <nil>\n. <nil>\ntrue invalid path\n"
    );
}

#[test]
fn compiles_and_runs_filepath_glob() {
    let source = r#"
package main
import "fmt"
import "path/filepath"
import "strings"

func main() {
    matches, err := filepath.Glob("assets/*.txt")
    nested, nestedErr := filepath.Glob("assets/nested/*.txt")
    none, noneErr := filepath.Glob("assets/*.md")
    _, bad := filepath.Glob("[")
    fmt.Println(strings.Join(matches, ","), err == nil)
    fmt.Println(strings.Join(nested, ","), nestedErr == nil)
    fmt.Println(none == nil, noneErr == nil, bad != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files.insert("assets/a.txt".into(), "a".into());
    vm.workspace_files.insert("assets/b.txt".into(), "b".into());
    vm.workspace_files
        .insert("assets/nested/c.txt".into(), "c".into());
    vm.workspace_files
        .insert("assets/keep.go".into(), "package keep".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "assets/a.txt,assets/b.txt true\nassets/nested/c.txt true\ntrue true true\n"
    );
}

#[test]
fn compiles_and_runs_filepath_abs() {
    let source = r#"
package main
import "fmt"
import "path/filepath"

func main() {
    rel, relErr := filepath.Abs("assets/a.txt")
    cwd, cwdErr := filepath.Abs(".")
    parent, parentErr := filepath.Abs("../a")
    rooted, rootedErr := filepath.Abs("/assets/../b")
    empty, emptyErr := filepath.Abs("")
    fmt.Println(rel, relErr == nil)
    fmt.Println(cwd, cwdErr == nil)
    fmt.Println(parent, parentErr == nil)
    fmt.Println(rooted, rootedErr == nil)
    fmt.Println(empty, emptyErr == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "/assets/a.txt true\n/ true\n/a true\n/b true\n/ true\n"
    );
}

#[test]
fn compiles_and_runs_filepath_walk_dir() {
    let source = r#"
package main
import "fmt"
import "io/fs"
import "path/filepath"
import "strings"

func main() {
    var seen []string
    err := filepath.WalkDir("assets", func(path string, d fs.DirEntry, err error) error {
        if err != nil {
            fmt.Println(path, d == nil, err != nil)
            return err
        }
        seen = append(seen, path)
        if path == "assets/skip" {
            return filepath.SkipDir
        }
        return nil
    })
    missingErr := filepath.WalkDir("missing", func(path string, d fs.DirEntry, err error) error {
        fmt.Println(path, d == nil, err != nil)
        return err
    })
    fmt.Println(strings.Join(seen, ","), err == nil)
    fmt.Println(missingErr != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files.insert("assets/a.txt".into(), "a".into());
    vm.workspace_files.insert("assets/z.txt".into(), "z".into());
    vm.workspace_files
        .insert("assets/skip/child.txt".into(), "child".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "missing true true\nassets,assets/a.txt,assets/skip,assets/z.txt true\ntrue\n"
    );
}

#[test]
fn compiles_and_runs_filepath_walk() {
    let source = r#"
package main
import "fmt"
import "io/fs"
import "path/filepath"
import "strings"

func main() {
    var seen []string
    err := filepath.Walk("assets", func(path string, info fs.FileInfo, err error) error {
        if err != nil {
            fmt.Println(path, info == nil, err != nil)
            return err
        }
        seen = append(seen, path)
        if path == "assets/skip" {
            return filepath.SkipDir
        }
        return nil
    })
    missingErr := filepath.Walk("missing", func(path string, info fs.FileInfo, err error) error {
        fmt.Println(path, info == nil, err != nil)
        return err
    })
    fmt.Println(strings.Join(seen, ","), err == nil)
    fmt.Println(missingErr != nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files.insert("assets/a.txt".into(), "a".into());
    vm.workspace_files.insert("assets/z.txt".into(), "z".into());
    vm.workspace_files
        .insert("assets/skip/child.txt".into(), "child".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "missing true true\nassets,assets/a.txt,assets/skip,assets/z.txt true\ntrue\n"
    );
}

#[test]
fn compiles_and_runs_filepath_browser_workspace_glob_and_walk_with_absolute_roots() {
    let source = r#"
package main
import "fmt"
import "io/fs"
import "path/filepath"
import "strings"

func main() {
    root, rootErr := filepath.Abs("assets/../assets")
    matches, matchesErr := filepath.Glob("/assets/../assets/*.txt")
    var walked []string
    walkErr := filepath.Walk("/assets/../assets", func(path string, info fs.FileInfo, err error) error {
        if err != nil {
            return err
        }
        walked = append(walked, path)
        if path == root+"/nested" {
            return filepath.SkipDir
        }
        return nil
    })
    fmt.Println(root, rootErr == nil)
    fmt.Println(strings.Join(matches, ","), matchesErr == nil)
    fmt.Println(strings.Join(walked, ","), walkErr == nil)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.workspace_files.insert("assets/a.txt".into(), "a".into());
    vm.workspace_files.insert("assets/b.txt".into(), "b".into());
    vm.workspace_files
        .insert("assets/nested/c.txt".into(), "c".into());
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "/assets true\n/assets/a.txt,/assets/b.txt true\n/assets,/assets/a.txt,/assets/b.txt,/assets/nested true\n"
    );
}

#[derive(Debug, Deserialize)]
struct PathDifferentialCorpusIndex {
    schema_version: u32,
    cases: Vec<PathDifferentialCase>,
}

#[derive(Debug, Deserialize)]
struct PathDifferentialCase {
    id: String,
    name: String,
    entry_path: String,
    workspace_files: Vec<String>,
    expected_native_go_stdout: String,
}

struct PathWorkspaceFileFixture {
    path: String,
    contents: String,
}

#[test]
fn path_differential_corpus_matches_checked_in_native_go_outputs() {
    let index = load_path_corpus_index();
    assert_eq!(
        index.schema_version, 1,
        "unexpected path/filepath differential schema"
    );
    assert!(
        !index.cases.is_empty(),
        "path/filepath differential corpus should contain representative cases"
    );

    for case in index.cases {
        run_path_case(case);
    }
}

fn load_path_corpus_index() -> PathDifferentialCorpusIndex {
    serde_json::from_str(
        &fs::read_to_string(path_corpus_root().join("index.json"))
            .expect("path/filepath differential corpus index should be readable"),
    )
    .expect("path/filepath differential corpus index should deserialize")
}

fn path_corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testdata/path-differential")
}

fn run_path_case(case: PathDifferentialCase) {
    let workspace_files = load_path_workspace_files(&case.id, &case.workspace_files);
    let source_inputs = workspace_files
        .iter()
        .filter(|file| file.path.ends_with(".go"))
        .map(|file| SourceInput {
            path: file.path.as_str(),
            source: file.contents.as_str(),
        })
        .collect::<Vec<_>>();

    let program = compile_workspace(&source_inputs, &case.entry_path).unwrap_or_else(|_| {
        panic!(
            "path/filepath differential case `{}` should compile",
            case.name
        )
    });

    let mut vm = Vm::new();
    vm.run_program(&program)
        .unwrap_or_else(|_| panic!("path/filepath differential case `{}` should run", case.name));
    assert_eq!(
        vm.stdout(),
        case.expected_native_go_stdout,
        "path/filepath differential case `{}` diverged from the checked-in native Go output",
        case.name
    );
}

fn load_path_workspace_files(
    case_id: &str,
    workspace_files: &[String],
) -> Vec<PathWorkspaceFileFixture> {
    let workspace_root = path_corpus_root().join(case_id).join("workspace");
    let mut files = Vec::with_capacity(workspace_files.len());
    for path in workspace_files {
        let full_path = workspace_root.join(path);
        files.push(PathWorkspaceFileFixture {
            path: path.clone(),
            contents: fs::read_to_string(&full_path).unwrap_or_else(|_| {
                panic!(
                    "path/filepath differential workspace file `{}` should be readable",
                    full_path.display()
                )
            }),
        });
    }
    files
}
