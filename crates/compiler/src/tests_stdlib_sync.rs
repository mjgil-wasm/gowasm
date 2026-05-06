use super::compile_source;
use gowasm_vm::Vm;

#[test]
fn sync_wait_group_waits_for_goroutine_completion() {
    let source = r#"
package main
import "fmt"
import "sync"

func main() {
    var wg sync.WaitGroup
    ch := make(chan int, 1)

    wg.Add(1)
    go func() {
        defer wg.Done()
        ch <- 7
    }()

    wg.Wait()
    fmt.Println(<-ch)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "7\n");
}

#[test]
fn sync_wait_group_add_negative_panics() {
    let source = r#"
package main
import "sync"

func main() {
    var wg sync.WaitGroup
    wg.Done()
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should panic");
    assert!(error
        .to_string()
        .contains("sync: negative WaitGroup counter"));
}

#[test]
fn sync_wait_group_wait_deadlocks_when_counter_never_reaches_zero() {
    let source = r#"
package main
import "sync"

func main() {
    var wg sync.WaitGroup
    wg.Add(1)
    wg.Wait()
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm
        .run_program(&program)
        .expect_err("wait group should deadlock");
    assert!(error.to_string().contains("all goroutines are blocked"));
}

#[test]
fn sync_once_runs_callback_only_once() {
    let source = r#"
package main
import "fmt"
import "sync"

func main() {
    var once sync.Once
    count := 0

    once.Do(func() {
        count = count + 1
        fmt.Println("ran", count)
    })
    once.Do(func() {
        count = count + 1
        fmt.Println("ran", count)
    })

    fmt.Println("count", count)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "ran 1\ncount 1\n");
}

#[test]
fn sync_once_marks_done_after_panic() {
    let source = r#"
package main
import "fmt"
import "sync"

func main() {
    var once sync.Once
    count := 0

    run := func(tag string) {
        defer func() {
            fmt.Println(tag, recover() != nil, count)
        }()
        once.Do(func() {
            count = count + 1
            panic("boom")
        })
        fmt.Println(tag, "after", count)
    }

    run("first")
    run("second")
    fmt.Println("final", count)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "first true 1\nsecond after 1\nsecond false 1\nfinal 1\n"
    );
}

#[test]
fn sync_once_panic_unblocks_waiters_without_rerunning_callback() {
    let source = r#"
package main
import "fmt"
import "sync"

func main() {
    var once sync.Once
    started := make(chan bool)
    release := make(chan bool)
    firstDone := make(chan bool)
    secondDone := make(chan bool)
    count := 0

    go func() {
        defer func() {
            _ = recover()
            firstDone <- true
        }()
        once.Do(func() {
            count = count + 1
            started <- true
            <-release
            panic("boom")
        })
    }()

    <-started

    go func() {
        once.Do(func() {
            count = count + 100
        })
        secondDone <- true
    }()

    release <- true
    <-firstDone
    <-secondDone
    fmt.Println(count)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1\n");
}

#[test]
fn sync_once_blocks_other_goroutines_until_callback_finishes() {
    let source = r#"
package main
import "fmt"
import "sync"

func main() {
    var once sync.Once
    count := 0
    started := make(chan bool)
    ready := make(chan bool)
    done := make(chan bool, 2)

    go func() {
        once.Do(func() {
            started <- true
            <-ready
            count = count + 1
        })
        done <- true
    }()

    <-started

    go func() {
        once.Do(func() {
            count = count + 100
        })
        done <- true
    }()

    ready <- true
    <-done
    <-done
    fmt.Println(count)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1\n");
}

#[test]
fn sync_mutex_blocks_competing_goroutines_until_unlock() {
    let source = r#"
package main
import "fmt"
import "sync"

func main() {
    var mu sync.Mutex
    started := make(chan bool)
    release := make(chan bool)
    done := make(chan bool, 2)

    go func() {
        mu.Lock()
        fmt.Println("first")
        started <- true
        <-release
        mu.Unlock()
        done <- true
    }()

    <-started

    go func() {
        mu.Lock()
        fmt.Println("second")
        mu.Unlock()
        done <- true
    }()

    fmt.Println("between")
    release <- true
    <-done
    <-done
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "first\nbetween\nsecond\n");
}

#[test]
fn sync_mutex_wakes_waiters_in_fifo_order() {
    let source = r#"
package main
import "fmt"
import "sync"

func main() {
    var mu sync.Mutex
    order := make(chan int, 2)
    ready1 := make(chan bool)
    ready2 := make(chan bool)

    mu.Lock()

    go func() {
        ready1 <- true
        mu.Lock()
        order <- 1
        mu.Unlock()
    }()

    <-ready1

    go func() {
        ready2 <- true
        mu.Lock()
        order <- 2
        mu.Unlock()
    }()

    <-ready2
    mu.Unlock()
    fmt.Println(<-order, <-order)
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1 2\n");
}

#[test]
fn sync_mutex_unlocks_through_defer() {
    let source = r#"
package main
import "fmt"
import "sync"

func main() {
    var mu sync.Mutex

    func() {
        mu.Lock()
        defer mu.Unlock()
        fmt.Println("inner")
    }()

    mu.Lock()
    fmt.Println("outer")
    mu.Unlock()
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "inner\nouter\n");
}

#[test]
fn sync_mutex_unlock_of_unlocked_mutex_panics() {
    let source = r#"
package main
import "sync"

func main() {
    var mu sync.Mutex
    mu.Unlock()
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should panic");
    assert!(error.to_string().contains("sync: unlock of unlocked mutex"));
}

#[test]
fn sync_rw_mutex_allows_parallel_readers() {
    let source = r#"
package main
import "fmt"
import "sync"

func main() {
    var mu sync.RWMutex
    done := make(chan bool)

    mu.RLock()
    go func() {
        mu.RLock()
        fmt.Println("second")
        mu.RUnlock()
        done <- true
    }()

    fmt.Println("first")
    <-done
    mu.RUnlock()
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "first\nsecond\n");
}

#[test]
fn sync_rw_mutex_writer_waits_for_readers_to_unlock() {
    let source = r#"
package main
import "fmt"
import "sync"

func main() {
    var mu sync.RWMutex
    started := make(chan bool)
    release := make(chan bool)
    done := make(chan bool)

    go func() {
        mu.RLock()
        fmt.Println("reader")
        started <- true
        <-release
        mu.RUnlock()
    }()

    <-started

    go func() {
        mu.Lock()
        fmt.Println("writer")
        mu.Unlock()
        done <- true
    }()

    fmt.Println("between")
    release <- true
    <-done
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "reader\nbetween\nwriter\n");
}

#[test]
fn sync_rw_mutex_waiting_writer_blocks_new_readers() {
    let source = r#"
package main
import "fmt"
import "sync"

func main() {
    var mu sync.RWMutex
    writerQueued := make(chan bool)
    done := make(chan bool, 2)

    mu.RLock()

    go func() {
        writerQueued <- true
        mu.Lock()
        fmt.Println("writer")
        mu.Unlock()
        done <- true
    }()

    <-writerQueued

    go func() {
        mu.RLock()
        fmt.Println("reader")
        mu.RUnlock()
        done <- true
    }()

    fmt.Println("release")
    mu.RUnlock()
    <-done
    <-done
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "release\nwriter\nreader\n");
}

#[test]
fn sync_rw_mutex_reader_waits_for_writer_to_unlock() {
    let source = r#"
package main
import "fmt"
import "sync"

func main() {
    var mu sync.RWMutex
    started := make(chan bool)
    release := make(chan bool)
    done := make(chan bool, 2)

    go func() {
        mu.Lock()
        fmt.Println("writer")
        started <- true
        <-release
        mu.Unlock()
        done <- true
    }()

    <-started

    go func() {
        mu.RLock()
        fmt.Println("reader")
        mu.RUnlock()
        done <- true
    }()

    fmt.Println("between")
    release <- true
    <-done
    <-done
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "writer\nbetween\nreader\n");
}

#[test]
fn sync_rw_mutex_unlock_of_unlocked_mutex_panics() {
    let source = r#"
package main
import "sync"

func main() {
    var mu sync.RWMutex
    mu.Unlock()
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should panic");
    assert!(error
        .to_string()
        .contains("sync: Unlock of unlocked RWMutex"));
}

#[test]
fn sync_rw_mutex_runlock_of_unlocked_mutex_panics() {
    let source = r#"
package main
import "sync"

func main() {
    var mu sync.RWMutex
    mu.RUnlock()
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    let error = vm.run_program(&program).expect_err("program should panic");
    assert!(error
        .to_string()
        .contains("sync: RUnlock of unlocked RWMutex"));
}
