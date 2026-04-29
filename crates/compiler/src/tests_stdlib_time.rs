use super::compile_source;
use gowasm_vm::{CapabilityRequest, RunOutcome, Vm};

#[test]
fn time_now_uses_the_host_provided_wall_clock() {
    let source = r#"
package main
import "fmt"
import "time"

func main() {
    now := time.Now()
    fmt.Println(now.Unix())
    fmt.Println(now.UnixMilli())
    fmt.Println(now.UnixNano())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.set_time_now_override_unix_nanos(1_700_000_000_123_000_000);
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "1700000000\n1700000000123\n1700000000123000000\n"
    );
}

#[test]
fn time_unix_and_unix_milli_round_trip_through_unix_methods() {
    let source = r#"
package main
import "fmt"
import "time"

func main() {
    fmt.Println(time.Unix(123, 456000000).UnixMilli())
    fmt.Println(time.UnixMilli(-1500).Unix())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "123456\n-2\n");
}

#[test]
fn time_unix_micro_and_comparison_methods_work() {
    let source = r#"
package main
import "fmt"
import "time"

func main() {
    earlier := time.UnixMicro(1234567)
    later := time.Unix(2, 0)

    fmt.Println(earlier.UnixMicro())
    fmt.Println(earlier.UnixNano())
    fmt.Println(earlier.Before(later), later.After(earlier))
    fmt.Println(earlier.Equal(later), earlier.Equal(time.UnixMicro(1234567)))
    fmt.Println(earlier.Compare(later), later.Compare(earlier), earlier.Compare(time.UnixMicro(1234567)))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "1234567\n1234567000\ntrue true\nfalse true\n-1 1 0\n"
    );
}

#[test]
fn time_time_methods_work_for_zero_values_and_pointers() {
    let source = r#"
package main
import "fmt"
import "time"

func main() {
    var zero time.Time
    value := time.UnixMilli(2500)
    pointer := &value

    fmt.Println(zero.Unix(), zero.UnixMilli(), zero.IsZero())
    fmt.Println(pointer.Unix(), pointer.UnixMilli(), pointer.IsZero())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "0 0 true\n2 2500 false\n");
}

#[test]
fn time_duration_constants_zero_values_and_methods_work() {
    let source = r#"
package main
import "fmt"
import "time"

func main() {
    var zero time.Duration
    converted := time.Duration(1500000000)

    fmt.Println(zero.Nanoseconds(), zero.Seconds())
    fmt.Println(time.Second.Nanoseconds(), time.Millisecond.Microseconds())
    fmt.Println(time.Minute.Seconds(), time.Hour.Minutes())
    fmt.Println(converted.Milliseconds(), converted.Seconds())
    fmt.Println(time.Duration(-1500).Microseconds())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "0 0.0\n1000000000 1000\n60.0 60.0\n1500 1.5\n-1\n"
    );
}

#[test]
fn time_since_until_add_and_sub_work_with_duration_values() {
    let source = r#"
package main
import "fmt"
import "time"

func main() {
    start := time.Unix(8, 500000000)
    end := time.Unix(12, 250000000)
    shifted := start.Add(time.Duration(1250000000))

    fmt.Println(time.Since(start).Milliseconds())
    fmt.Println(time.Until(end).Milliseconds())
    fmt.Println(shifted.UnixMilli())
    fmt.Println(end.Sub(start).Milliseconds())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.set_time_now_override_unix_nanos(10_000_000_000);
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1500\n2250\n9750\n3750\n");
}

#[test]
fn time_sleep_requests_a_host_timer_and_resumes_execution() {
    let source = r#"
package main
import "fmt"
import "time"

func main() {
    time.Sleep(2000000)
    fmt.Println("after sleep")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.enable_capability_requests();

    match vm.start_program(&program).expect("program should start") {
        RunOutcome::CapabilityRequest(CapabilityRequest::Sleep { duration_nanos }) => {
            assert_eq!(duration_nanos, 2_000_000);
        }
        other => panic!("unexpected run outcome: {other:?}"),
    }

    vm.advance_timers(&program, 2_000_000, None)
        .expect("timers should advance");
    match vm.resume_program(&program).expect("program should resume") {
        RunOutcome::Completed => {}
        other => panic!("unexpected run outcome: {other:?}"),
    }
    assert_eq!(vm.stdout(), "after sleep\n");
}

#[test]
fn time_after_delivers_a_time_value_after_host_timer_resume() {
    let source = r#"
package main
import "fmt"
import "time"

func main() {
    fired := <-time.After(2000000)
    fmt.Println(fired.UnixMilli())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.enable_capability_requests();

    match vm.start_program(&program).expect("program should start") {
        RunOutcome::CapabilityRequest(CapabilityRequest::Sleep { duration_nanos }) => {
            assert_eq!(duration_nanos, 2_000_000);
        }
        other => panic!("unexpected run outcome: {other:?}"),
    }

    vm.advance_timers(&program, 2_000_000, Some(1_700_000_000_789_000_000))
        .expect("timers should advance");
    match vm.resume_program(&program).expect("program should resume") {
        RunOutcome::Completed => {}
        other => panic!("unexpected run outcome: {other:?}"),
    }
    assert_eq!(vm.stdout(), "1700000000789\n");
}

#[test]
fn time_after_zero_duration_delivers_immediately_with_current_time() {
    let source = r#"
package main
import "fmt"
import "time"

func main() {
    fired := <-time.After(0)
    fmt.Println(fired.UnixMilli())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.set_time_now_override_unix_nanos(1_700_000_000_123_000_000);
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1700000000123\n");
}

#[test]
fn time_new_timer_delivers_a_time_value_via_timer_channel() {
    let source = r#"
package main
import "fmt"
import "time"

func main() {
    timer := time.NewTimer(2000000)
    fired := <-timer.C
    fmt.Println(fired.UnixMilli())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.enable_capability_requests();

    match vm.start_program(&program).expect("program should start") {
        RunOutcome::CapabilityRequest(CapabilityRequest::Sleep { duration_nanos }) => {
            assert_eq!(duration_nanos, 2_000_000);
        }
        other => panic!("unexpected run outcome: {other:?}"),
    }

    vm.advance_timers(&program, 2_000_000, Some(1_700_000_000_987_000_000))
        .expect("timers should advance");
    match vm.resume_program(&program).expect("program should resume") {
        RunOutcome::Completed => {}
        other => panic!("unexpected run outcome: {other:?}"),
    }
    assert_eq!(vm.stdout(), "1700000000987\n");
}

#[test]
fn time_new_timer_zero_duration_delivers_immediately_with_current_time() {
    let source = r#"
package main
import "fmt"
import "time"

func main() {
    timer := time.NewTimer(0)
    fired := <-timer.C
    fmt.Println(fired.UnixMilli())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.set_time_now_override_unix_nanos(1_700_000_000_321_000_000);
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "1700000000321\n");
}

#[test]
fn time_timer_stop_cancels_a_pending_timer_before_the_vm_pauses() {
    let source = r#"
package main
import "fmt"
import "time"

func main() {
    timer := time.NewTimer(2000000)
    fmt.Println(timer.Stop())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "true\n");
}

#[test]
fn time_timer_stop_returns_false_after_the_timer_fires() {
    let source = r#"
package main
import "fmt"
import "time"

func main() {
    timer := time.NewTimer(2000000)
    <-timer.C
    fmt.Println(timer.Stop())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.enable_capability_requests();

    match vm.start_program(&program).expect("program should start") {
        RunOutcome::CapabilityRequest(CapabilityRequest::Sleep { duration_nanos }) => {
            assert_eq!(duration_nanos, 2_000_000);
        }
        other => panic!("unexpected run outcome: {other:?}"),
    }

    vm.advance_timers(&program, 2_000_000, Some(1_700_000_001_111_000_000))
        .expect("timers should advance");
    match vm.resume_program(&program).expect("program should resume") {
        RunOutcome::Completed => {}
        other => panic!("unexpected run outcome: {other:?}"),
    }
    assert_eq!(vm.stdout(), "false\n");
}

#[test]
fn time_timer_reset_returns_true_and_rearms_an_active_timer() {
    let source = r#"
package main
import "fmt"
import "time"

func main() {
    timer := time.NewTimer(5000000)
    fmt.Println(timer.Reset(2000000))
    fired := <-timer.C
    fmt.Println(fired.UnixMilli())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.enable_capability_requests();

    match vm.start_program(&program).expect("program should start") {
        RunOutcome::CapabilityRequest(CapabilityRequest::Sleep { duration_nanos }) => {
            assert_eq!(duration_nanos, 2_000_000);
        }
        other => panic!("unexpected run outcome: {other:?}"),
    }

    vm.advance_timers(&program, 2_000_000, Some(1_700_000_001_222_000_000))
        .expect("timers should advance");
    match vm.resume_program(&program).expect("program should resume") {
        RunOutcome::Completed => {}
        other => panic!("unexpected run outcome: {other:?}"),
    }
    assert_eq!(vm.stdout(), "true\n1700000001222\n");
}

#[test]
fn time_timer_reset_returns_false_after_stop_and_reschedules_the_timer() {
    let source = r#"
package main
import "fmt"
import "time"

func main() {
    timer := time.NewTimer(2000000)
    fmt.Println(timer.Stop())
    fmt.Println(timer.Reset(3000000))
    fired := <-timer.C
    fmt.Println(fired.UnixMilli())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.enable_capability_requests();

    match vm.start_program(&program).expect("program should start") {
        RunOutcome::CapabilityRequest(CapabilityRequest::Sleep { duration_nanos }) => {
            assert_eq!(duration_nanos, 3_000_000);
        }
        other => panic!("unexpected run outcome: {other:?}"),
    }

    vm.advance_timers(&program, 3_000_000, Some(1_700_000_001_333_000_000))
        .expect("timers should advance");
    match vm.resume_program(&program).expect("program should resume") {
        RunOutcome::Completed => {}
        other => panic!("unexpected run outcome: {other:?}"),
    }
    assert_eq!(vm.stdout(), "true\nfalse\n1700000001333\n");
}

#[test]
fn time_timer_reset_drains_a_stale_buffered_value_before_rearming() {
    let source = r#"
package main
import "fmt"
import "time"

func main() {
    timer := time.NewTimer(2000000)
    time.Sleep(3000000)
    fmt.Println(timer.Reset(4000000))
    fired := <-timer.C
    fmt.Println(fired.UnixMilli())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.enable_capability_requests();

    match vm.start_program(&program).expect("program should start") {
        RunOutcome::CapabilityRequest(CapabilityRequest::Sleep { duration_nanos }) => {
            assert_eq!(duration_nanos, 2_000_000);
        }
        other => panic!("unexpected run outcome: {other:?}"),
    }

    vm.advance_timers(&program, 2_000_000, Some(1_700_000_001_444_000_000))
        .expect("timers should advance");
    match vm.resume_program(&program).expect("program should resume") {
        RunOutcome::CapabilityRequest(CapabilityRequest::Sleep { duration_nanos }) => {
            assert_eq!(duration_nanos, 1_000_000);
        }
        other => panic!("unexpected run outcome: {other:?}"),
    }

    vm.advance_timers(&program, 1_000_000, Some(1_700_000_001_445_000_000))
        .expect("sleep should finish");
    match vm.resume_program(&program).expect("program should resume") {
        RunOutcome::CapabilityRequest(CapabilityRequest::Sleep { duration_nanos }) => {
            assert_eq!(duration_nanos, 4_000_000);
        }
        other => panic!("unexpected run outcome: {other:?}"),
    }

    vm.advance_timers(&program, 4_000_000, Some(1_700_000_001_449_000_000))
        .expect("rearmed timer should fire");
    match vm.resume_program(&program).expect("program should resume") {
        RunOutcome::Completed => {}
        other => panic!("unexpected run outcome: {other:?}"),
    }
    assert_eq!(vm.stdout(), "false\n1700000001449\n");
}

#[test]
fn time_timer_stop_and_reset_survive_repeated_cancellation_and_rearming() {
    let source = r#"
package main
import "fmt"
import "time"

func main() {
    timer := time.NewTimer(5000000)
    fmt.Println(timer.Stop())
    fmt.Println(timer.Reset(4000000))
    fmt.Println(timer.Stop())
    fmt.Println(timer.Reset(3000000))
    fired := <-timer.C
    fmt.Println(fired.UnixMilli())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.enable_capability_requests();

    match vm.start_program(&program).expect("program should start") {
        RunOutcome::CapabilityRequest(CapabilityRequest::Sleep { duration_nanos }) => {
            assert_eq!(duration_nanos, 3_000_000);
        }
        other => panic!("unexpected run outcome: {other:?}"),
    }

    vm.advance_timers(&program, 3_000_000, Some(1_700_000_001_555_000_000))
        .expect("timers should advance");
    match vm.resume_program(&program).expect("program should resume") {
        RunOutcome::Completed => {}
        other => panic!("unexpected run outcome: {other:?}"),
    }
    assert_eq!(vm.stdout(), "true\nfalse\ntrue\nfalse\n1700000001555\n");
}

#[test]
fn time_sleep_only_pauses_when_the_scheduler_goes_idle() {
    let source = r#"
package main
import "fmt"
import "time"

func main() {
    go func() {
        fmt.Println("worker ready")
    }()
    time.Sleep(1000000)
    fmt.Println("main done")
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.enable_capability_requests();

    match vm.start_program(&program).expect("program should start") {
        RunOutcome::CapabilityRequest(CapabilityRequest::Sleep { duration_nanos }) => {
            assert_eq!(duration_nanos, 1_000_000);
        }
        other => panic!("unexpected run outcome: {other:?}"),
    }
    assert_eq!(vm.stdout(), "worker ready\n");

    vm.advance_timers(&program, 1_000_000, None)
        .expect("timers should advance");
    match vm.resume_program(&program).expect("program should resume") {
        RunOutcome::Completed => {}
        other => panic!("unexpected run outcome: {other:?}"),
    }
    assert_eq!(vm.stdout(), "worker ready\nmain done\n");
}

#[test]
fn time_datetime_constant_and_format_render_numeric_utc_layouts() {
    let source = r#"
package main
import "fmt"
import "time"

func main() {
    value := time.Unix(1700000000, 0)
    fmt.Println(time.DateTime)
    fmt.Println(time.ANSIC)
    fmt.Println(time.RFC850)
    fmt.Println(time.RFC1123)
    fmt.Println(time.RFC1123Z)
    fmt.Println(time.RFC3339)
    fmt.Println(value.Format(time.DateTime))
    fmt.Println(value.Format("15:04:05 2006/01/02"))
    fmt.Println(time.Unix(-1, 0).Format(time.DateTime))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "2006-01-02 15:04:05\nMon Jan _2 15:04:05 2006\nMonday, 02-Jan-06 15:04:05 MST\nMon, 02 Jan 2006 15:04:05 MST\nMon, 02 Jan 2006 15:04:05 -0700\n2006-01-02T15:04:05Z07:00\n2023-11-14 22:13:20\n22:13:20 2023/11/14\n1969-12-31 23:59:59\n"
    );
}

#[test]
fn time_parse_supports_http_related_layouts() {
    let source = r#"
package main
import "fmt"
import "net/http"
import "time"

func main() {
    parsed, err := time.Parse(http.TimeFormat, "Sun, 06 Nov 1994 08:49:37 GMT")
    fmt.Println(err == nil, parsed.Format(time.DateTime))

    parsed, err = time.Parse(time.RFC850, "Sunday, 06-Nov-94 08:49:37 GMT")
    fmt.Println(err == nil, parsed.Format(time.DateTime))

    parsed, err = time.Parse(time.ANSIC, "Sun Nov  6 08:49:37 1994")
    fmt.Println(err == nil, parsed.Format(time.DateTime))

    parsed, err = time.Parse(time.RFC1123, "Sun, 06 Nov 1994 08:49:37 GMT")
    fmt.Println(err == nil, parsed.Format(time.DateTime))

    parsed, err = time.Parse(time.RFC1123Z, "Sun, 06 Nov 1994 01:49:37 -0700")
    fmt.Println(err == nil, parsed.Format(time.DateTime))

    parsed, err = time.Parse(time.RFC3339, "1994-11-06T08:49:37Z")
    fmt.Println(err == nil, parsed.Format(time.DateTime))

    parsed, err = time.Parse(time.RFC3339, "1994-11-06T01:49:37-07:00")
    fmt.Println(err == nil, parsed.Format(time.DateTime))

    parsed, err = time.Parse(time.DateTime, "1994-11-06 08:49:37")
    fmt.Println(err != nil, parsed.IsZero())
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "true 1994-11-06 08:49:37\ntrue 1994-11-06 08:49:37\ntrue 1994-11-06 08:49:37\ntrue 1994-11-06 08:49:37\ntrue 1994-11-06 08:49:37\ntrue 1994-11-06 08:49:37\ntrue 1994-11-06 08:49:37\ntrue true\n"
    );
}

#[test]
fn time_format_supports_named_utc_layouts() {
    let source = r#"
package main
import "fmt"
import "net/http"
import "time"

func main() {
    value := time.Unix(1700000000, 0)
    fmt.Println(value.Format(time.ANSIC))
    fmt.Println(value.Format(time.RFC850))
    fmt.Println(value.Format(time.RFC1123))
    fmt.Println(value.Format(time.RFC1123Z))
    fmt.Println(value.Format(time.RFC3339))
    fmt.Println(value.Format(http.TimeFormat))
    fmt.Println(time.Unix(-1, 0).Format(time.RFC1123Z))
    fmt.Println(time.Unix(-1, 0).Format(time.RFC3339))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(
        vm.stdout(),
        "Tue Nov 14 22:13:20 2023\nTuesday, 14-Nov-23 22:13:20 UTC\nTue, 14 Nov 2023 22:13:20 UTC\nTue, 14 Nov 2023 22:13:20 +0000\n2023-11-14T22:13:20Z\nTue, 14 Nov 2023 22:13:20 GMT\nWed, 31 Dec 1969 23:59:59 +0000\n1969-12-31T23:59:59Z\n"
    );
}


#[test]
fn time_zero_value_timer_methods_handle_missing_channel() {
    let source = r#"
package main
import "fmt"
import "time"

func main() {
    var t time.Timer
    fmt.Println(t.Stop())
    fmt.Println(t.Reset(100))
}
"#;

    let program = compile_source(source).expect("program should compile");
    let mut vm = Vm::new();
    vm.run_program(&program).expect("program should run");
    assert_eq!(vm.stdout(), "false\nfalse\n");
}
