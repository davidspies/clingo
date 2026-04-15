use std::ffi::CStr;
use std::os::raw::{c_char, c_void};

/// Warning codes from the clingo logger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Warning {
    OperationUndefined,
    RuntimeError,
    AtomUndefined,
    FileIncluded,
    VariableUnbounded,
    GlobalVariable,
    Other,
}

impl Warning {
    pub(crate) fn from_raw(code: clingo_sys::clingo_warning_t) -> Self {
        match code as u32 {
            clingo_sys::clingo_warning_e_clingo_warning_operation_undefined => {
                Warning::OperationUndefined
            }
            clingo_sys::clingo_warning_e_clingo_warning_runtime_error => Warning::RuntimeError,
            clingo_sys::clingo_warning_e_clingo_warning_atom_undefined => Warning::AtomUndefined,
            clingo_sys::clingo_warning_e_clingo_warning_file_included => Warning::FileIncluded,
            clingo_sys::clingo_warning_e_clingo_warning_variable_unbounded => {
                Warning::VariableUnbounded
            }
            clingo_sys::clingo_warning_e_clingo_warning_global_variable => Warning::GlobalVariable,
            _ => Warning::Other,
        }
    }
}

/// C-compatible trampoline for the logger callback.
pub(crate) unsafe extern "C" fn logger_trampoline(
    code: clingo_sys::clingo_warning_t,
    message: *const c_char,
    data: *mut c_void,
) {
    let closure = unsafe { &mut *(data as *mut Box<dyn FnMut(Warning, &str)>) };
    let warning = Warning::from_raw(code);
    let msg = unsafe { CStr::from_ptr(message) }
        .to_str()
        .unwrap_or("<invalid UTF-8>");
    closure(warning, msg);
}
