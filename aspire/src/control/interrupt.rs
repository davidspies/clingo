use std::sync::{Arc, RwLock};

/// Wrapper around a raw pointer that opts into `Send + Sync`.
///
/// Safety: the pointee (`clingo_control_t`) supports `clingo_control_interrupt`
/// from any thread, and we guard access with an `RwLock` to prevent use-after-free.
pub(crate) struct RawControlPtr(pub(crate) *mut clingo_sys::clingo_control_t);
unsafe impl Send for RawControlPtr {}
unsafe impl Sync for RawControlPtr {}

/// A thread-safe handle that can interrupt a running solve.
///
/// Obtained via [`Control::interrupt_handle`](super::Control::interrupt_handle).
/// The handle is `Send + Sync` and can be cloned, so it can be shared across threads.
///
/// If the `Control` has been dropped, `interrupt()` is a safe no-op.
#[derive(Clone)]
pub struct InterruptHandle {
    pub(crate) ptr: Arc<RwLock<RawControlPtr>>,
}

impl InterruptHandle {
    /// Interrupt the running solve operation.
    ///
    /// This is safe to call from any thread. If the `Control` has already
    /// been dropped, this is a no-op.
    pub fn interrupt(&self) {
        let guard = self.ptr.read().unwrap();
        if !guard.0.is_null() {
            unsafe {
                clingo_sys::clingo_control_interrupt(guard.0);
            }
        }
    }
}
