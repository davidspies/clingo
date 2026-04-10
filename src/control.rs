use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

use crate::error::{Error, check};
use crate::solve::{Model, SolveResult, solve_with_callback};
use crate::symbol::Symbol;

/// Owns a `clingo_control_t` and frees it on drop.
///
/// This is the main entry point for grounding and solving logic programs.
pub struct Control {
    pub(crate) ptr: ptr::NonNull<clingo_sys::clingo_control_t>,
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
        let c_args: Vec<CString> = args
            .iter()
            .map(|s| CString::new(*s))
            .collect::<Result<_, _>>()?;
        let c_ptrs: Vec<*const c_char> = c_args.iter().map(|s| s.as_ptr()).collect();

        let mut raw: *mut clingo_sys::clingo_control_t = ptr::null_mut();
        check(unsafe {
            clingo_sys::clingo_control_new(
                c_ptrs.as_ptr(),
                c_ptrs.len(),
                None, // default logger (stderr)
                ptr::null_mut(),
                20, // default message limit
                &mut raw,
            )
        })?;

        Ok(Control {
            ptr: ptr::NonNull::new(raw).expect("clingo_control_new returned null without error"),
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
}
