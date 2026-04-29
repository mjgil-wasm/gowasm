use super::{
    resolve_stdlib_method, stdlib_function_param_types, stdlib_function_result_count,
    stdlib_function_result_types, stdlib_function_returns_value,
};

#[test]
fn resolves_sync_methods_from_the_registry() {
    let add = resolve_stdlib_method("*sync.WaitGroup", "Add")
        .expect("(*sync.WaitGroup).Add should exist");
    assert!(!stdlib_function_returns_value(add));
    assert_eq!(stdlib_function_result_count(add), 0);
    assert_eq!(
        stdlib_function_param_types(add),
        Some(&["*sync.WaitGroup", "int"][..])
    );
    assert_eq!(stdlib_function_result_types(add), Some(&[][..]));

    let wait = resolve_stdlib_method("*sync.WaitGroup", "Wait")
        .expect("(*sync.WaitGroup).Wait should exist");
    assert!(!stdlib_function_returns_value(wait));
    assert_eq!(
        stdlib_function_param_types(wait),
        Some(&["*sync.WaitGroup"][..])
    );
    assert_eq!(stdlib_function_result_types(wait), Some(&[][..]));

    let once_do = resolve_stdlib_method("*sync.Once", "Do").expect("(*sync.Once).Do should exist");
    assert!(!stdlib_function_returns_value(once_do));
    assert_eq!(stdlib_function_result_count(once_do), 0);
    assert_eq!(
        stdlib_function_param_types(once_do),
        Some(&["*sync.Once", "__gowasm_func__()->()"][..])
    );
    assert_eq!(stdlib_function_result_types(once_do), Some(&[][..]));

    let mutex_lock =
        resolve_stdlib_method("*sync.Mutex", "Lock").expect("(*sync.Mutex).Lock should exist");
    assert!(!stdlib_function_returns_value(mutex_lock));
    assert_eq!(stdlib_function_result_count(mutex_lock), 0);
    assert_eq!(
        stdlib_function_param_types(mutex_lock),
        Some(&["*sync.Mutex"][..])
    );
    assert_eq!(stdlib_function_result_types(mutex_lock), Some(&[][..]));

    let mutex_unlock =
        resolve_stdlib_method("*sync.Mutex", "Unlock").expect("(*sync.Mutex).Unlock should exist");
    assert!(!stdlib_function_returns_value(mutex_unlock));
    assert_eq!(stdlib_function_result_count(mutex_unlock), 0);
    assert_eq!(
        stdlib_function_param_types(mutex_unlock),
        Some(&["*sync.Mutex"][..])
    );
    assert_eq!(stdlib_function_result_types(mutex_unlock), Some(&[][..]));

    let rw_mutex_lock =
        resolve_stdlib_method("*sync.RWMutex", "Lock").expect("(*sync.RWMutex).Lock should exist");
    assert!(!stdlib_function_returns_value(rw_mutex_lock));
    assert_eq!(stdlib_function_result_count(rw_mutex_lock), 0);
    assert_eq!(
        stdlib_function_param_types(rw_mutex_lock),
        Some(&["*sync.RWMutex"][..])
    );
    assert_eq!(stdlib_function_result_types(rw_mutex_lock), Some(&[][..]));

    let rw_mutex_rlock = resolve_stdlib_method("*sync.RWMutex", "RLock")
        .expect("(*sync.RWMutex).RLock should exist");
    assert!(!stdlib_function_returns_value(rw_mutex_rlock));
    assert_eq!(stdlib_function_result_count(rw_mutex_rlock), 0);
    assert_eq!(
        stdlib_function_param_types(rw_mutex_rlock),
        Some(&["*sync.RWMutex"][..])
    );
    assert_eq!(stdlib_function_result_types(rw_mutex_rlock), Some(&[][..]));

    let rw_mutex_runlock = resolve_stdlib_method("*sync.RWMutex", "RUnlock")
        .expect("(*sync.RWMutex).RUnlock should exist");
    assert!(!stdlib_function_returns_value(rw_mutex_runlock));
    assert_eq!(stdlib_function_result_count(rw_mutex_runlock), 0);
    assert_eq!(
        stdlib_function_param_types(rw_mutex_runlock),
        Some(&["*sync.RWMutex"][..])
    );
    assert_eq!(
        stdlib_function_result_types(rw_mutex_runlock),
        Some(&[][..])
    );
}
