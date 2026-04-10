use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr;

use crate::config::Configuration;
use crate::error::{ClingoError, Error, check};
use crate::solve::{Model, SolveResult, solve_with_callback};
use crate::symbol::Symbol;

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
    fn from_raw(code: clingo_sys::clingo_warning_t) -> Self {
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
unsafe extern "C" fn logger_trampoline(
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

/// Owns a `clingo_control_t` and frees it on drop.
///
/// This is the main entry point for grounding and solving logic programs.
pub struct Control {
    pub(crate) ptr: ptr::NonNull<clingo_sys::clingo_control_t>,
    // Boxed logger closure, kept alive for the lifetime of the Control.
    // Must be dropped *after* ptr since clingo may call the logger during cleanup.
    _logger: Option<Box<Box<dyn FnMut(Warning, &str)>>>,
}

// clingo_control_t is single-threaded; do not send across threads.

impl Drop for Control {
    fn drop(&mut self) {
        unsafe {
            clingo_sys::clingo_control_free(self.ptr.as_ptr());
        }
    }
}

impl Control {
    /// Create a new control object.
    ///
    /// `args` are command-line style options forwarded to the grounder/solver
    /// (e.g. `["0"]` for enumerating all models).
    pub fn new(args: &[&str]) -> Result<Self, Error> {
        Self::with_logger(args, None::<fn(Warning, &str)>, 20)
    }

    /// Create a new control object with a custom logger.
    ///
    /// The logger receives a `Warning` code and message string.
    /// `message_limit` controls how many messages are reported
    /// before being suppressed (0 for unlimited).
    pub fn with_logger(
        args: &[&str],
        logger: Option<impl FnMut(Warning, &str) + 'static>,
        message_limit: u32,
    ) -> Result<Self, Error> {
        let c_args: Vec<CString> = args
            .iter()
            .map(|s| CString::new(*s))
            .collect::<Result<_, _>>()?;
        let c_ptrs: Vec<*const c_char> = c_args.iter().map(|s| s.as_ptr()).collect();

        let (logger_fn, logger_data, boxed_logger) = match logger {
            Some(f) => {
                let mut boxed: Box<Box<dyn FnMut(Warning, &str)>> = Box::new(Box::new(f));
                let data = &mut *boxed as *mut Box<dyn FnMut(Warning, &str)> as *mut c_void;
                (Some(logger_trampoline as _), data, Some(boxed))
            }
            None => (None, ptr::null_mut(), None),
        };

        let mut raw: *mut clingo_sys::clingo_control_t = ptr::null_mut();
        check(unsafe {
            clingo_sys::clingo_control_new(
                c_ptrs.as_ptr(),
                c_ptrs.len(),
                logger_fn,
                logger_data,
                message_limit as std::os::raw::c_uint,
                &mut raw,
            )
        })?;

        Ok(Control {
            ptr: ptr::NonNull::new(raw).expect("clingo_control_new returned null without error"),
            _logger: boxed_logger,
        })
    }

    /// Add a logic program block.
    ///
    /// `name` is the program section name (typically `"base"`).
    /// `parameters` are the parameter names of the program block
    /// (empty for `#program base.`).
    /// `program` is the ASP source text.
    pub fn add(&mut self, name: &str, parameters: &[&str], program: &str) -> Result<(), Error> {
        let c_name = CString::new(name)?;
        let c_params: Vec<CString> = parameters
            .iter()
            .map(|s| CString::new(*s))
            .collect::<Result<_, _>>()?;
        let c_param_ptrs: Vec<*const c_char> = c_params.iter().map(|s| s.as_ptr()).collect();
        let c_program = CString::new(program)?;

        check(unsafe {
            clingo_sys::clingo_control_add(
                self.ptr.as_ptr(),
                c_name.as_ptr(),
                c_param_ptrs.as_ptr(),
                c_param_ptrs.len(),
                c_program.as_ptr(),
            )
        })?;
        Ok(())
    }

    /// Ground the given program parts.
    ///
    /// Each element is `(name, &[symbol_values])`. For the common case of
    /// grounding just the `"base"` part with no parameters, use
    /// [`ground_base`](Self::ground_base).
    ///
    /// No external-function callback is supported yet — pass programs that
    /// don't use `@`-functions.
    pub fn ground(&mut self, parts: &[(&str, &[Symbol])]) -> Result<(), Error> {
        let c_names: Vec<CString> = parts
            .iter()
            .map(|(name, _)| CString::new(*name))
            .collect::<Result<_, _>>()?;

        let raw_params: Vec<Vec<clingo_sys::clingo_symbol_t>> = parts
            .iter()
            .map(|(_, params)| params.iter().map(|s| s.0).collect())
            .collect();

        let c_parts: Vec<clingo_sys::clingo_part_t> = c_names
            .iter()
            .zip(raw_params.iter())
            .map(|(c_name, params)| clingo_sys::clingo_part_t {
                name: c_name.as_ptr(),
                params: params.as_ptr(),
                size: params.len(),
            })
            .collect();

        check(unsafe {
            clingo_sys::clingo_control_ground(
                self.ptr.as_ptr(),
                c_parts.as_ptr(),
                c_parts.len(),
                None,
                ptr::null_mut(),
            )
        })?;
        Ok(())
    }

    /// Convenience: ground just the `"base"` part with no parameters.
    pub fn ground_base(&mut self) -> Result<(), Error> {
        self.ground(&[("base", &[])])
    }

    /// Solve the program, calling `on_model` for each model found.
    ///
    /// The callback returns `Ok(true)` to continue searching or `Ok(false)` to stop.
    pub fn solve(
        &mut self,
        on_model: impl FnMut(&Model) -> Result<bool, Error>,
    ) -> Result<SolveResult, Error> {
        solve_with_callback(self.ptr.as_ptr(), on_model)
    }

    /// Get the configuration object for reading and modifying solver settings.
    pub fn configuration(&mut self) -> Result<Configuration<'_>, ClingoError> {
        let mut cfg: *mut clingo_sys::clingo_configuration_t = ptr::null_mut();
        check(unsafe { clingo_sys::clingo_control_configuration(self.ptr.as_ptr(), &mut cfg) })?;
        let mut root: clingo_sys::clingo_id_t = 0;
        check(unsafe { clingo_sys::clingo_configuration_root(cfg, &mut root) })?;
        Ok(unsafe { Configuration::new(cfg, root) })
    }

    /// Look up the program literal for a ground atom symbol.
    ///
    /// Returns `None` if the symbol doesn't appear in the ground program.
    fn literal_for_symbol(
        &self,
        symbol: Symbol,
    ) -> Result<Option<clingo_sys::clingo_literal_t>, ClingoError> {
        let mut atoms: *const clingo_sys::clingo_symbolic_atoms_t = ptr::null();
        check(unsafe { clingo_sys::clingo_control_symbolic_atoms(self.ptr.as_ptr(), &mut atoms) })?;

        let mut iter: clingo_sys::clingo_symbolic_atom_iterator_t = 0;
        check(unsafe { clingo_sys::clingo_symbolic_atoms_find(atoms, symbol.raw(), &mut iter) })?;

        let mut end: clingo_sys::clingo_symbolic_atom_iterator_t = 0;
        check(unsafe { clingo_sys::clingo_symbolic_atoms_end(atoms, &mut end) })?;

        let mut equal = false;
        check(unsafe {
            clingo_sys::clingo_symbolic_atoms_iterator_is_equal_to(atoms, iter, end, &mut equal)
        })?;

        if equal {
            return Ok(None);
        }

        let mut literal: clingo_sys::clingo_literal_t = 0;
        check(unsafe { clingo_sys::clingo_symbolic_atoms_literal(atoms, iter, &mut literal) })?;

        Ok(Some(literal))
    }

    /// Assign a truth value to an external atom.
    ///
    /// The atom must have been declared with `#external` in the program.
    /// Returns `Ok(false)` if the symbol doesn't appear in the ground program.
    pub fn assign_external(&mut self, symbol: Symbol, value: TruthValue) -> Result<bool, Error> {
        let Some(literal) = self.literal_for_symbol(symbol)? else {
            return Ok(false);
        };
        check(unsafe {
            clingo_sys::clingo_control_assign_external(self.ptr.as_ptr(), literal, value.to_raw())
        })?;
        Ok(true)
    }

    /// Release an external atom, making it no longer external.
    ///
    /// After this, the atom is subject to normal program simplification.
    /// Returns `Ok(false)` if the symbol doesn't appear in the ground program.
    pub fn release_external(&mut self, symbol: Symbol) -> Result<bool, Error> {
        let Some(literal) = self.literal_for_symbol(symbol)? else {
            return Ok(false);
        };
        check(unsafe { clingo_sys::clingo_control_release_external(self.ptr.as_ptr(), literal) })?;
        Ok(true)
    }
}

/// Truth value for external atoms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TruthValue {
    /// Let the solver decide.
    Free,
    /// Assign true.
    True,
    /// Assign false.
    False,
}

impl TruthValue {
    fn to_raw(self) -> clingo_sys::clingo_truth_value_t {
        match self {
            TruthValue::Free => clingo_sys::clingo_truth_value_e_clingo_truth_value_free as i32,
            TruthValue::True => clingo_sys::clingo_truth_value_e_clingo_truth_value_true as i32,
            TruthValue::False => clingo_sys::clingo_truth_value_e_clingo_truth_value_false as i32,
        }
    }
}
