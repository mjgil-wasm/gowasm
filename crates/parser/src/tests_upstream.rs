use super::parse_source_file;

fn assert_upstream_parses(source: &str) {
    parse_source_file(source).expect("source should parse");
}

#[test]
fn parses_upstream_kaval_grouped_type_declarations() {
    // Source: github.com/kaval-lang/kaval (0BSD), adapted from parser.go.
    let source = r#"
package kaval

import "fmt"

type ParserEvent interface {
    isParserEvent()
}

type (
    ListStartEvent struct{}
    ListEndEvent struct{}
    MapStartEvent struct{}
    MapEndEvent struct{}
    MapKeyEvent struct {
        Value
    }
    ValueEvent struct {
        Value
    }
    ErrorEvent struct {
        Pos Position
        Msg string
    }
)

func (e ErrorEvent) Error() string {
    return fmt.Sprintf("Error at %s: %s", e.Pos, e.Msg)
}

func (e ValueEvent) Unwrap() Value {
    return e.Value
}

func (ValueEvent) isParserEvent()     {}
func (ListStartEvent) isParserEvent() {}
func (ListEndEvent) isParserEvent()   {}
func (MapStartEvent) isParserEvent()  {}
func (MapEndEvent) isParserEvent()    {}
func (MapKeyEvent) isParserEvent()    {}
func (ErrorEvent) isParserEvent()     {}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_kaval_anonymous_interface_type_assertions() {
    // Source: github.com/kaval-lang/kaval (0BSD), adapted from value.go.
    let source = r#"
package kaval

type Value interface {
    Raw() string
}

func As[T any](v any) (val T, ok bool) {
    val, ok = v.(T)
    if ok {
        return val, true
    }

    conv, ok := v.(interface{ Unwrap() Value })
    if ok {
        return As[T](conv.Unwrap())
    }

    var zero T
    return zero, false
}

func IsNil(v Value) bool {
    check, ok := As[interface{ IsNil() bool }](v)
    if ok {
        return check.IsNil()
    }
    return false
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_td_logger_types() {
    // Source: github.com/voioo/td (0BSD), adapted from internal/logger/logger.go.
    let source = r#"
package logger

import (
    "fmt"
    "io"
    "strings"
    "time"
)

type Level int

const (
    LevelDebug Level = iota
    LevelInfo
    LevelWarn
    LevelError
    LevelFatal
)

type Logger struct {
    level  Level
    writer io.Writer
    json   bool
}

type Field struct {
    Key   string
    Value interface{}
}

type LogEntry struct {
    Time    time.Time              `json:"time"`
    Level   string                 `json:"level"`
    Message string                 `json:"message"`
    Fields  map[string]interface{} `json:"fields,omitempty"`
}

func (l *Logger) log(level Level, message string, fields map[string]interface{}) {
    if level < l.level {
        return
    }

    entry := LogEntry{
        Time:    time.Now().UTC(),
        Level:   "INFO",
        Message: message,
        Fields:  fields,
    }

    var sb strings.Builder
    sb.WriteString(fmt.Sprintf("%s [%s] %s", entry.Time.Format("2006-01-02"), entry.Level, entry.Message))
}

func fieldsToMap(fields []Field) map[string]interface{} {
    if len(fields) == 0 {
        return nil
    }

    m := make(map[string]interface{}, len(fields))
    for _, f := range fields {
        m[f.Key] = f.Value
    }
    return m
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_govalid_object_validator_shapes() {
    // Source: github.com/gima/govalid (Unlicense), adapted from v1/object.go.
    let source = r#"
package govalid

import "reflect"

type Validator interface {
    Validate(data interface{}) (string, error)
}

type ObjectOpt func(m *reflect.Value) (string, error)

func ObjValues(v Validator) ObjectOpt {
    return func(m *reflect.Value) (string, error) {
        for _, key := range m.MapKeys() {
            value := m.MapIndex(key).Interface()
            if path, err := v.Validate(value); err != nil {
                return path, err
            }
        }
        return "", nil
    }
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_kaval_builder_options_and_errors() {
    // Source: github.com/kaval-lang/kaval (0BSD), adapted from builder.go.
    let source = r#"
package kaval

import "fmt"

var ErrOddNumberOfPairs = fmt.Errorf("odd number of pairs")
var ErrOrderedFieldAfterLabeled = fmt.Errorf("ordered field after labeled field")

type BuilderOptions struct {
    AlwaysQuoteStrings          bool
    SpaceAfterFieldSeparator    bool
    SpaceAfterListSeparator     bool
    SpaceAfterPairsSeparator    bool
    SpaceAroundFieldAssignment  bool
}

type Builder struct {
    options    BuilderOptions
    fields     []string
    hasLabeled bool
    nextLabel  string
    err        error
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_kaval_builder_lists_and_labels() {
    // Source: github.com/kaval-lang/kaval (0BSD), adapted from builder.go.
    let source = r#"
package kaval

import "strings"

type Builder struct {
    fields []string
}

func (b *Builder) List(values ...any) *Builder {
    items := make([]string, len(values))
    for i, v := range values {
        items[i] = strings.TrimSpace(v.(string))
    }
    return &Builder{fields: items}
}

func (b *Builder) Dict(pairs ...any) *Builder {
    items := make([]string, 0, len(pairs)/2)
    for i := 0; i < len(pairs); i += 2 {
        items = append(items, pairs[i].(string)+":"+pairs[i+1].(string))
    }
    b.fields = append(b.fields, strings.Join(items, ";"))
    return b
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_kaval_character_classifiers() {
    // Source: github.com/kaval-lang/kaval (0BSD), adapted from classify.go.
    let source = r#"
package kaval

import "unicode"

func isSpace(ch rune) bool {
    return ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r'
}

func isHexDigit(ch rune) bool {
    return (ch >= '0' && ch <= '9') || (ch >= 'a' && ch <= 'f') || (ch >= 'A' && ch <= 'F')
}

func isIdentifierContinue(ch rune) bool {
    return unicode.IsLetter(ch) || unicode.IsDigit(ch) || ch == '-' || ch == '_'
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_kaval_position_stringer() {
    // Source: github.com/kaval-lang/kaval (0BSD), adapted from position.go.
    let source = r#"
package kaval

import "fmt"

type Position struct {
    Offset int
    Column int
}

func (p Position) String() string {
    return fmt.Sprintf("Col %d (Offset %d)", p.Column, p.Offset)
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_td_keymap_nested_binding_slices() {
    // Source: github.com/voioo/td (0BSD), adapted from internal/ui/keymap.go.
    let source = r#"
package ui

import "github.com/charmbracelet/bubbles/key"

type KeyMap struct {
    Add            key.Binding
    Delete         key.Binding
    Up             key.Binding
    Down           key.Binding
    Left           key.Binding
    Right          key.Binding
    Edit           key.Binding
    ListType       key.Binding
    Filter         key.Binding
    Escape         key.Binding
    Help           key.Binding
    Quit           key.Binding
    Undo           key.Binding
    Redo           key.Binding
    PriorityNone   key.Binding
    PriorityLow    key.Binding
    PriorityMedium key.Binding
    PriorityHigh   key.Binding
    Home           key.Binding
    End            key.Binding
    ClearCompleted key.Binding
}

func (k KeyMap) FullHelp() [][]key.Binding {
    return [][]key.Binding{
        []key.Binding{k.Add, k.Delete, k.Up, k.Down, k.Left, k.Right, k.Edit},
        []key.Binding{k.ListType, k.Filter, k.Escape},
        []key.Binding{k.Help, k.Quit, k.Undo, k.Redo},
        []key.Binding{k.PriorityNone, k.PriorityLow, k.PriorityMedium, k.PriorityHigh},
        []key.Binding{k.Home, k.End, k.ClearCompleted},
    }
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_td_ui_validation_table_tests() {
    // Source: github.com/voioo/td (0BSD), adapted from internal/ui/validation_test.go.
    let source = r#"
package ui

import "testing"

func TestValidateTaskName(t *testing.T) {
    tests := []struct {
        name     string
        input    string
        expected error
    }{
        {"valid name", "Buy groceries", nil},
        {"empty name", "", nil},
        {"valid unicode", "Comprar", nil},
    }

    for _, test := range tests {
        t.Run(test.name, func(t *testing.T) {
            err := ValidateTaskName(test.input)
            if test.expected == nil && err != nil {
                t.Errorf("expected no error, got %v", err)
            }
        })
    }
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_td_memory_repository() {
    // Source: github.com/voioo/td (0BSD), adapted from internal/storage/memory.go.
    let source = r#"
package storage

import (
    "sync"
    "github.com/voioo/td/internal/task"
)

type MemoryRepository struct {
    mu          sync.RWMutex
    activeTasks []*task.Task
    doneTasks   []*task.Task
    nextID      int
}

var _ TaskRepository = (*MemoryRepository)(nil)

func (r *MemoryRepository) SaveTasks(activeTasks []*task.Task, doneTasks []*task.Task) error {
    r.mu.Lock()
    defer r.mu.Unlock()

    r.activeTasks = make([]*task.Task, len(activeTasks))
    copy(r.activeTasks, activeTasks)
    r.doneTasks = make([]*task.Task, len(doneTasks))
    copy(r.doneTasks, doneTasks)
    return nil
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_td_task_manager_methods() {
    // Source: github.com/voioo/td (0BSD), adapted from internal/task/task.go.
    let source = r#"
package task

import (
    "sync"
    "time"
)

type Priority int

const (
    PriorityNone Priority = iota
    PriorityLow
    PriorityMedium
    PriorityHigh
)

type Task struct {
    CreatedAt time.Time
    Name      string
    ID        int
    IsDone    bool
    Priority  Priority
}

type TaskManager struct {
    tasks     []*Task
    doneTasks []*Task
    nextID    int
    mu        sync.Mutex
}

func (tm *TaskManager) AddTask(name string) *Task {
    tm.mu.Lock()
    defer tm.mu.Unlock()
    tm.nextID = tm.nextID + 1
    task := &Task{ID: tm.nextID, Name: name, CreatedAt: time.Now()}
    tm.tasks = append(tm.tasks, task)
    return task
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_td_storage_file_repository() {
    // Source: github.com/voioo/td (0BSD), adapted from internal/storage/storage.go.
    let source = r#"
package storage

import (
    "encoding/json"
    "fmt"
    "os"
    "strings"
    "time"
    "github.com/voioo/td/internal/task"
)

type FileRepository struct {
    filePath string
}

func (r *FileRepository) validateTask(t *task.Task) error {
    if t == nil {
        return fmt.Errorf("task is nil")
    }
    if strings.TrimSpace(t.Name) == "" {
        return fmt.Errorf("task name cannot be empty")
    }
    if t.CreatedAt.After(time.Now().Add(24 * time.Hour)) {
        return fmt.Errorf("task creation time cannot be more than 24 hours in the future")
    }
    return nil
}

func (r *FileRepository) SaveTasks(tasks []*task.Task) error {
    data, err := json.MarshalIndent(tasks, "", "  ")
    if err != nil {
        return err
    }
    _, err = os.Stdout.Write(data)
    return err
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_td_cmd_main_flags() {
    // Source: github.com/voioo/td (0BSD), adapted from cmd/td/main.go.
    let source = r#"
package main

import (
    "flag"
    "fmt"
    "os"
)

var version = "dev"
var commit = "none"
var date = "unknown"

func main() {
    versionFlag := flag.Bool("version", false, "print version information")
    flag.BoolVar(versionFlag, "v", false, "print version information (shorthand)")
    flag.Parse()

    if *versionFlag {
        fmt.Printf("td %s (commit: %s, built at: %s)\n", version, commit, date)
        os.Exit(0)
    }
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_go_commons_worker_pool() {
    // Source: github.com/Rodert/go-commons (Unlicense), adapted from concurrentutils.go.
    let source = r#"
package concurrentutils

import (
    "context"
    "sync"
)

type WorkerPool struct {
    workers   int
    taskQueue chan func()
    wg        sync.WaitGroup
    ctx       context.Context
}

func (wp *WorkerPool) worker() {
    defer wp.wg.Done()
    for {
        select {
        case <-wp.ctx.Done():
            return
        case task, ok := <-wp.taskQueue:
            if !ok {
                return
            }
            if task != nil {
                task()
            }
        }
    }
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_go_commons_rate_limiter() {
    // Source: github.com/Rodert/go-commons (Unlicense), adapted from concurrentutils.go.
    let source = r#"
package concurrentutils

import (
    "sync"
    "time"
)

type RateLimiter struct {
    limit    int64
    interval time.Duration
    tokens   int64
    lastTime int64
    mu       sync.Mutex
}

func (rl *RateLimiter) Allow() bool {
    rl.mu.Lock()
    defer rl.mu.Unlock()
    now := time.Now().UnixNano()
    elapsedSeconds := float64(now-rl.lastTime) / float64(time.Second)
    tokensToAdd := int64(elapsedSeconds * float64(rl.limit))
    if tokensToAdd > 0 {
        rl.tokens = rl.tokens + tokensToAdd
        rl.lastTime = now
    }
    return rl.tokens > 0
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_go_commons_slice_helpers() {
    // Source: github.com/Rodert/go-commons (Unlicense), adapted from sliceutils.go.
    let source = r#"
package sliceutils

import "reflect"

func Unique(slice interface{}) []interface{} {
    v := reflect.ValueOf(slice)
    if v.Kind() != reflect.Slice {
        return nil
    }
    seen := make(map[interface{}]bool)
    result := make([]interface{}, 0, v.Len())
    for i := 0; i < v.Len(); i++ {
        item := v.Index(i).Interface()
        if !seen[item] {
            seen[item] = true
            result = append(result, item)
        }
    }
    return result
}

func FilterInt(slice []int, fn func(int) bool) []int {
    result := make([]int, 0, len(slice))
    for _, item := range slice {
        if fn(item) {
            result = append(result, item)
        }
    }
    return result
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_go_commons_string_transformers() {
    // Source: github.com/Rodert/go-commons (Unlicense), adapted from string_transform.go.
    let source = r#"
package stringutils

import (
    "strings"
    "unicode"
    "unicode/utf8"
)

func Reverse(str string) string {
    runes := []rune(str)
    for i, j := 0, len(runes)-1; i < j; i, j = i+1, j-1 {
        runes[i], runes[j] = runes[j], runes[i]
    }
    return string(runes)
}

func PadCenter(str string, size int, padChar rune) string {
    runeCount := utf8.RuneCountInString(str)
    if runeCount >= size {
        return str
    }
    if unicode.IsUpper(padChar) {
        return strings.Repeat(string(padChar), size-runeCount) + str
    }
    return str
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_go_commons_concurrent_example_main() {
    // Source: github.com/Rodert/go-commons (Unlicense), adapted from examples/concurrentutils/main.go.
    let source = r#"
package main

import (
    "context"
    "fmt"
    "sync"
    "sync/atomic"
    "time"
)

func main() {
    var counter int64
    var wg sync.WaitGroup
    tasks := 3

    for i := 0; i < tasks; i++ {
        wg.Add(1)
        taskID := i
        go func() {
            defer wg.Done()
            atomic.AddInt64(&counter, 1)
            fmt.Printf("task %d\n", taskID)
        }()
    }

    ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
    defer cancel()
    _, _ = counter, ctx
    wg.Wait()
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_govalid_optional_validator() {
    // Source: github.com/gima/govalid (Unlicense), adapted from optional.go.
    let source = r#"
package govalid

import (
    "fmt"
    "reflect"
)

type Validator interface {
    Validate(data interface{}) (string, error)
}

type optionalValidator struct {
    validator Validator
}

func (r *optionalValidator) Validate(data interface{}) (string, error) {
    if data == nil {
        return "", nil
    }
    typ := reflect.TypeOf(data)
    if typ.Kind() == reflect.Ptr && reflect.ValueOf(data).IsNil() {
        return "", nil
    }
    if path, err := r.validator.Validate(data); err != nil {
        return fmt.Sprintf("Optional->%s", path), err
    }
    return "", nil
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_govalid_function_validator() {
    // Source: github.com/gima/govalid (Unlicense), adapted from function.go.
    let source = r#"
package govalid

type Validator interface {
    Validate(data interface{}) (string, error)
}

type ValidatorFunc func(data interface{}) (string, error)
type functionValidator struct {
    validatorfunc ValidatorFunc
}

func Function(validatorfunc ValidatorFunc) Validator {
    return &functionValidator{validatorfunc}
}

func (r *functionValidator) Validate(data interface{}) (string, error) {
    return r.validatorfunc(data)
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_govalid_array_validator() {
    // Source: github.com/gima/govalid (Unlicense), adapted from array.go.
    let source = r#"
package govalid

import (
    "fmt"
    "reflect"
)

type ArrayOpt func(slice *reflect.Value) (string, error)

func ArrEach(validator Validator) ArrayOpt {
    return func(slice *reflect.Value) (string, error) {
        for i := 0; i < slice.Len(); i++ {
            path, err := validator.Validate(slice.Index(i).Interface())
            if err != nil {
                return path, fmt.Errorf("idx %d: %s", i, err.Error())
            }
        }
        return "", nil
    }
}

type arrayValidator struct {
    opts []ArrayOpt
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_go_kata_roman_numeral_encoder() {
    // Source: github.com/J-R-Oliver/go-kata (Unlicense), adapted from romannumeralencoder.go.
    let source = r#"
package kata

func RomanNumeralEncoder(i int) string {
    iRemaining := i
    romanNumeral := ""

    if iRemaining >= 10 {
        for ; iRemaining >= 10; iRemaining -= 10 {
            romanNumeral += "X"
        }
        iRemaining %= 10
    }

    for ; iRemaining >= 1; iRemaining-- {
        romanNumeral += "I"
    }

    return romanNumeral
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_go_kata_move_table_test() {
    // Source: github.com/J-R-Oliver/go-kata (Unlicense), adapted from move_test.go.
    let source = r#"
package kata

import "testing"

func TestMove(t *testing.T) {
    tests := []struct {
        name     string
        position int
        roll     int
        want     int
    }{
        {"start zero", 0, 4, 8},
        {"start three", 3, 6, 15},
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            if got := Move(tt.position, tt.roll); got != tt.want {
                t.Errorf("Move() = %v, want %v", got, tt.want)
            }
        })
    }
}
"#;

    assert_upstream_parses(source);
}

#[test]
fn parses_upstream_bull_http_error_wrapper() {
    // Source: github.com/gokrazy/bull (0BSD), adapted from internal/bull/httperr.go.
    let source = r#"
package bull

import (
    "context"
    "log"
    "net/http"
)

type httpErr struct {
    code int
    err  error
}

func (h *httpErr) Error() string {
    return h.err.Error()
}

func handleError(h func(http.ResponseWriter, *http.Request) error) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        err := h(w, r)
        if err == nil {
            return
        }
        if err == context.Canceled {
            return
        }
        if he, ok := err.(*httpErr); ok {
            log.Printf("%d %s", he.code, he.err)
        }
    })
}
"#;

    assert_upstream_parses(source);
}
