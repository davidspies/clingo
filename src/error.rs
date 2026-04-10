use std::ffi::{CStr, NulError};
use std::fmt;

/// Error codes returned by the clingo C library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClingoErrorCode {
    Runtime,
    Logic,
    BadAlloc,
    Unknown,
}

/// An error originating from the clingo C library.
#[derive(Debug, Clone)]
pub struct ClingoError {
    pub code: ClingoErrorCode,
    pub message: Option<String>,
}

impl fmt::Display for ClingoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.message {
            Some(msg) => write!(f, "clingo {:?}: {}", self.code, msg),
            None => write!(f, "clingo {:?}", self.code),
        }
    }
}

impl std::error::Error for ClingoError {}

/// Wrapper enum so callers get one error type.
#[derive(Debug)]
pub enum Error {
    Clingo(ClingoError),
    NulByte(NulError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Clingo(e) => e.fmt(f),
            Error::NulByte(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<NulError> for Error {
    fn from(e: NulError) -> Self {
        Error::NulByte(e)
    }
}

impl From<ClingoError> for Error {
    fn from(e: ClingoError) -> Self {
        Error::Clingo(e)
    }
}

/// Read the thread-local error state from clingo and return an `Err`.
fn last_clingo_error() -> ClingoError {
    unsafe {
        let code = clingo_sys::clingo_error_code();
        let msg_ptr = clingo_sys::clingo_error_message();
        let message = if msg_ptr.is_null() {
            None
        } else {
            Some(CStr::from_ptr(msg_ptr).to_string_lossy().into_owned())
        };
        let code = match code as u32 {
            clingo_sys::clingo_error_e_clingo_error_runtime => ClingoErrorCode::Runtime,
            clingo_sys::clingo_error_e_clingo_error_logic => ClingoErrorCode::Logic,
            clingo_sys::clingo_error_e_clingo_error_bad_alloc => ClingoErrorCode::BadAlloc,
            _ => ClingoErrorCode::Unknown,
        };
        ClingoError { code, message }
    }
}

/// Check a C bool return value; convert failure to `Result`.
pub(crate) fn check(ok: bool) -> Result<(), ClingoError> {
    if ok { Ok(()) } else { Err(last_clingo_error()) }
}
