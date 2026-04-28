use crate::ValueData;

#[test]
fn workspace_error_values_wrap_known_os_sentinels() {
    let missing = super::workspace_fs_impl::error_value("open missing: file does not exist");
    let invalid = super::workspace_fs_impl::error_value("open ../bad: invalid path");

    match &missing.data {
        ValueData::Error(error) => {
            assert_eq!(error.message, "open missing: file does not exist");
            assert_eq!(error.kind_message.as_deref(), Some("file does not exist"));
            let wrapped = error.wrapped.as_ref().expect("missing error should unwrap");
            match &wrapped.data {
                ValueData::Error(inner) => {
                    assert_eq!(inner.message, "open missing: file does not exist")
                }
                other => panic!("expected wrapped error value, got {other:?}"),
            }
        }
        other => panic!("expected error value, got {other:?}"),
    }

    match &invalid.data {
        ValueData::Error(error) => {
            assert_eq!(error.message, "open ../bad: invalid path");
            assert_eq!(error.kind_message.as_deref(), Some("invalid argument"));
            let wrapped = error.wrapped.as_ref().expect("invalid error should unwrap");
            match &wrapped.data {
                ValueData::Error(inner) => assert_eq!(inner.message, "invalid argument"),
                other => panic!("expected wrapped error value, got {other:?}"),
            }
        }
        other => panic!("expected error value, got {other:?}"),
    }
}

#[test]
fn workspace_error_values_leave_non_sentinel_messages_plain() {
    let value = super::workspace_fs_impl::error_value("mkdir keep.txt/child: not a directory");
    match &value.data {
        ValueData::Error(error) => {
            assert_eq!(error.message, "mkdir keep.txt/child: not a directory");
            assert!(error.kind_message.is_none());
            assert!(error.wrapped.is_none());
        }
        other => panic!("expected error value, got {other:?}"),
    }
}
