//! The only unsafe boundary in Nota. Handles are registry tokens, never dereferenced.
//! The host serializes calls and waits for a successful flush before destroying a handle.
#![deny(unsafe_op_in_unsafe_fn)]

use std::collections::HashMap;
use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use nota_app::session::{OpenOptionsJson, Request, Session, SessionError};
use serde_json::Value;

#[repr(C)]
pub struct NotaBuffer {
    pub ptr: *mut u8,
    pub len: usize,
}

impl NotaBuffer {
    fn empty() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
            len: 0,
        }
    }
    fn reply(value: Value) -> Self {
        let mut bytes = value.to_string().into_bytes().into_boxed_slice();
        let result = Self {
            ptr: bytes.as_mut_ptr(),
            len: bytes.len(),
        };
        std::mem::forget(bytes);
        result
    }
}

struct BindingSession {
    session: Session,
    faulted: bool,
}

type Sessions = HashMap<usize, Arc<Mutex<BindingSession>>>;
static SESSIONS: OnceLock<Mutex<Sessions>> = OnceLock::new();
static NEXT_HANDLE: AtomicUsize = AtomicUsize::new(1);

fn sessions() -> std::sync::MutexGuard<'static, Sessions> {
    SESSIONS
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

fn rejection(code: &'static str, message: impl ToString) -> NotaBuffer {
    NotaBuffer::reply(SessionError::new(code, message).reply())
}

#[unsafe(no_mangle)]
pub extern "C" fn nota_abi_version() -> u32 {
    1
}

/// # Safety
/// Non-null input must point to len readable bytes. Session and reply must
/// point to writable, aligned output slots. The host frees every returned buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nota_open(
    options: *const u8,
    len: usize,
    session: *mut *mut c_void,
    reply: *mut NotaBuffer,
) -> i32 {
    if session.is_null() || reply.is_null() {
        return 3;
    }
    // SAFETY: The caller supplies valid output slots as specified above.
    unsafe {
        *session = std::ptr::null_mut();
        *reply = NotaBuffer::empty();
    }
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: Input validity is the caller's contract; null and length are checked.
        let bytes = unsafe { input(options, len) }.map_err(|error| (3, error))?;
        let options: OpenOptionsJson = serde_json::from_slice(bytes)
            .map_err(|e| (3, SessionError::new("invalid_argument", e)))?;
        let opened = Session::open(options).map_err(|e| (1, e))?;
        let response = opened.reply(Value::Null);
        let handle = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
        if handle == 0 || handle == usize::MAX {
            return Err((
                1,
                SessionError::new("handle_exhausted", "Session handle space exhausted"),
            ));
        }
        let buffer = NotaBuffer::reply(response);
        sessions().insert(
            handle,
            Arc::new(Mutex::new(BindingSession {
                session: opened,
                faulted: false,
            })),
        );
        // SAFETY: The output slots are valid. The token is opaque and is never dereferenced.
        unsafe {
            *session = handle as *mut c_void;
            *reply = buffer;
        }
        Ok(())
    }));
    finish(outcome, reply)
}

/// # Safety
/// Request must point to len readable bytes and reply to a writable aligned
/// output slot. The handle must be a token returned by nota_open.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nota_execute(
    session: *mut c_void,
    request: *const u8,
    len: usize,
    reply: *mut NotaBuffer,
) -> i32 {
    if reply.is_null() {
        return 3;
    }
    // SAFETY: The caller supplies a valid output slot.
    unsafe {
        *reply = NotaBuffer::empty();
    }
    let binding = sessions().get(&(session as usize)).cloned();
    let Some(binding) = binding else {
        // SAFETY: The caller supplies a valid output slot.
        unsafe {
            *reply = rejection("invalid_handle", "Unknown or closed session");
        }
        return 3;
    };
    let mut binding = binding.lock().unwrap_or_else(|error| error.into_inner());
    if binding.faulted {
        // SAFETY: The caller supplies a valid output slot.
        unsafe {
            *reply = rejection("session_faulted", "Close and reopen the faulted session");
        }
        return 2;
    }
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: Input validity is the caller's contract; null and length are checked.
        let bytes = unsafe { input(request, len) }.map_err(|error| (3, error))?;
        let request: Request = serde_json::from_slice(bytes)
            .map_err(|e| (3, SessionError::new("invalid_argument", e)))?;
        let response = binding.session.execute(request).map_err(|e| (1, e))?;
        // SAFETY: The caller supplies a valid output slot.
        unsafe {
            *reply = NotaBuffer::reply(response);
        }
        Ok(())
    }));
    if outcome.is_err() {
        binding.faulted = true;
    }
    finish(outcome, reply)
}

type CallOutcome = Result<Result<(), (i32, SessionError)>, Box<dyn std::any::Any + Send>>;

fn finish(outcome: CallOutcome, reply: *mut NotaBuffer) -> i32 {
    let (status, buffer) = match outcome {
        Ok(Ok(())) => return 0,
        Ok(Err((status, error))) => (status, NotaBuffer::reply(error.reply())),
        Err(_) => (
            2,
            rejection("panic", "The native session encountered an internal error"),
        ),
    };
    // SAFETY: Both callers validate reply and require it to be a writable output slot.
    unsafe {
        *reply = buffer;
    }
    status
}

unsafe fn input<'a>(ptr: *const u8, len: usize) -> Result<&'a [u8], SessionError> {
    if ptr.is_null() || len == 0 || len > isize::MAX as usize {
        return Err(SessionError::new(
            "invalid_argument",
            "Input must contain a JSON document",
        ));
    }
    // SAFETY: The caller guarantees a readable allocation of len bytes.
    Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
}

/// # Safety
/// Pass only a buffer returned by this library, exactly once, without modifying
/// its pointer or length. An empty buffer is also accepted.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nota_buffer_free(buffer: NotaBuffer) {
    if buffer.ptr.is_null() {
        return;
    }
    // SAFETY: The buffer originated from a boxed slice in NotaBuffer::reply.
    unsafe {
        drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            buffer.ptr, buffer.len,
        )));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn nota_destroy(session: *mut c_void) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let removed = sessions().remove(&(session as usize));
        drop(removed);
    }));
}
