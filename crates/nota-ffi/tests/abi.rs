use nota_ffi::{
    NotaBuffer, nota_abi_version, nota_buffer_free, nota_destroy, nota_execute, nota_open,
};
use serde_json::{Value, json};
use std::ffi::c_void;

fn empty() -> NotaBuffer {
    NotaBuffer {
        ptr: std::ptr::null_mut(),
        len: 0,
    }
}

fn take(buffer: NotaBuffer) -> Value {
    assert!(!buffer.ptr.is_null());
    // SAFETY: Each buffer was just returned by the ABI and is consumed once.
    unsafe {
        let value =
            serde_json::from_slice(std::slice::from_raw_parts(buffer.ptr, buffer.len)).unwrap();
        nota_buffer_free(buffer);
        value
    }
}

fn open(path: &std::path::Path) -> *mut c_void {
    let options = json!({"data_directory":path}).to_string();
    let mut session = std::ptr::null_mut();
    let mut reply = empty();
    // SAFETY: Input and output pointers remain valid for the duration of this call.
    let status = unsafe { nota_open(options.as_ptr(), options.len(), &mut session, &mut reply) };
    let response = take(reply);
    assert_eq!(status, 0, "{response}");
    session
}

fn execute(session: *mut c_void, request: &[u8]) -> (i32, Value) {
    let mut reply = empty();
    // SAFETY: Input and output pointers remain valid for the duration of this call.
    let status = unsafe { nota_execute(session, request.as_ptr(), request.len(), &mut reply) };
    (status, take(reply))
}

#[test]
fn abi_round_trip_and_negative_inputs_leave_session_usable() {
    assert_eq!(nota_abi_version(), 1);
    let temp = tempfile::tempdir().unwrap();
    let session = open(temp.path());
    for bad in [
        b"{".as_slice(),
        &[0xff],
        br#"{"command":"unknown"}"#,
        br#"{"command":"select_note","id":"not-a-uuid"}"#,
    ] {
        let (status, reply) = execute(session, bad);
        assert_eq!(status, 3);
        assert_eq!(reply["ok"], false);
    }
    let mut reply = empty();
    // SAFETY: Null input is explicitly accepted as an invalid argument; reply is valid.
    assert_eq!(
        unsafe { nota_execute(session, std::ptr::null(), 8, &mut reply) },
        3
    );
    assert_eq!(take(reply)["error"]["code"], "invalid_argument");
    let (status, reply) = execute(session, br#"{"command":"new_note"}"#);
    assert_eq!(status, 0);
    assert_eq!(reply["snapshot"]["rows"].as_array().unwrap().len(), 1);
    assert_eq!(execute(session, br#"{"command":"flush"}"#).0, 0);
    nota_destroy(session);
    assert_eq!(execute(session, br#"{"command":"snapshot"}"#).0, 3);
    nota_destroy(session);
    let reopened = open(temp.path());
    assert_eq!(
        execute(reopened, br#"{"command":"snapshot"}"#).1["snapshot"]["rows"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    nota_destroy(reopened);
}

#[test]
fn open_rejects_relative_paths_and_initializes_outputs() {
    let options = br#"{"data_directory":"relative"}"#;
    let mut session = 123usize as *mut c_void;
    let mut reply = empty();
    // SAFETY: Input and output pointers remain valid for this call.
    assert_eq!(
        unsafe { nota_open(options.as_ptr(), options.len(), &mut session, &mut reply) },
        1
    );
    assert!(session.is_null());
    assert_eq!(take(reply)["error"]["code"], "invalid_path");
    // SAFETY: Null output slots are explicitly rejected.
    assert_eq!(
        unsafe {
            nota_open(
                options.as_ptr(),
                options.len(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        },
        3
    );
}
