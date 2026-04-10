use std::os::raw::c_void;
use std::ptr;

use crate::error::{ClingoError, Error, check};
use crate::symbol::Symbol;

/// The result of a solve call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SolveResult {
    pub satisfiable: bool,
    pub unsatisfiable: bool,
    pub exhausted: bool,
    pub interrupted: bool,
}

impl SolveResult {
    fn from_bitset(bits: u32) -> Self {
        SolveResult {
            satisfiable: bits & clingo_sys::clingo_solve_result_e_clingo_solve_result_satisfiable
                != 0,
            unsatisfiable: bits
                & clingo_sys::clingo_solve_result_e_clingo_solve_result_unsatisfiable
                != 0,
            exhausted: bits & clingo_sys::clingo_solve_result_e_clingo_solve_result_exhausted != 0,
            interrupted: bits & clingo_sys::clingo_solve_result_e_clingo_solve_result_interrupted
                != 0,
        }
    }
}

/// Which symbols to include when querying a model.
#[derive(Debug, Clone, Copy)]
pub enum ShowType {
    /// Atoms that are shown (via `#show`).
    Shown,
    /// All atoms.
    Atoms,
    /// All terms.
    Terms,
    /// Theory atoms.
    Theory,
    /// Everything.
    All,
}

impl ShowType {
    fn to_bitset(self) -> u32 {
        match self {
            ShowType::Shown => clingo_sys::clingo_show_type_e_clingo_show_type_shown,
            ShowType::Atoms => clingo_sys::clingo_show_type_e_clingo_show_type_atoms,
            ShowType::Terms => clingo_sys::clingo_show_type_e_clingo_show_type_terms,
            ShowType::Theory => clingo_sys::clingo_show_type_e_clingo_show_type_theory,
            ShowType::All => clingo_sys::clingo_show_type_e_clingo_show_type_all,
        }
    }
}

/// A model found during solving. Only valid for the duration of the callback.
pub struct Model {
    ptr: *const clingo_sys::clingo_model_t,
}

impl Model {
    pub(crate) fn from_ptr(ptr: *const clingo_sys::clingo_model_t) -> Self {
        Model { ptr }
    }

    /// Get the symbols in this model matching the given show type.
    pub fn symbols(&self, show: ShowType) -> Result<Vec<Symbol>, ClingoError> {
        let bits = show.to_bitset();
        let mut size: usize = 0;
        check(unsafe { clingo_sys::clingo_model_symbols_size(self.ptr, bits, &mut size) })?;
        let mut raw = vec![0u64; size];
        check(unsafe { clingo_sys::clingo_model_symbols(self.ptr, bits, raw.as_mut_ptr(), size) })?;
        Ok(raw
            .into_iter()
            .map(|s| unsafe { Symbol::from_raw(s) })
            .collect())
    }

    /// Check whether a specific atom is in the model.
    pub fn contains(&self, atom: Symbol) -> Result<bool, ClingoError> {
        let mut contained = false;
        check(unsafe { clingo_sys::clingo_model_contains(self.ptr, atom.raw(), &mut contained) })?;
        Ok(contained)
    }

    /// Get the running number of this model.
    pub fn number(&self) -> Result<u64, ClingoError> {
        let mut n: u64 = 0;
        check(unsafe { clingo_sys::clingo_model_number(self.ptr, &mut n) })?;
        Ok(n)
    }

    /// Whether the optimality of this model has been proven.
    pub fn optimality_proven(&self) -> Result<bool, ClingoError> {
        let mut proven = false;
        check(unsafe { clingo_sys::clingo_model_optimality_proven(self.ptr, &mut proven) })?;
        Ok(proven)
    }

    /// Get the cost vector of this model.
    pub fn cost(&self) -> Result<Vec<i64>, ClingoError> {
        let mut size: usize = 0;
        check(unsafe { clingo_sys::clingo_model_cost_size(self.ptr, &mut size) })?;
        let mut costs = vec![0i64; size];
        check(unsafe { clingo_sys::clingo_model_cost(self.ptr, costs.as_mut_ptr(), size) })?;
        Ok(costs)
    }
}

/// State passed through the C callback trampoline.
struct CallbackState<'a> {
    closure: &'a mut dyn FnMut(&Model) -> Result<bool, Error>,
    error: Option<Error>,
}

/// C-compatible trampoline that forwards to the Rust closure.
unsafe extern "C" fn solve_trampoline(
    type_: clingo_sys::clingo_solve_event_type_t,
    event: *mut c_void,
    data: *mut c_void,
    goon: *mut bool,
) -> bool {
    let state = unsafe { &mut *(data as *mut CallbackState) };

    if type_ != clingo_sys::clingo_solve_event_type_e_clingo_solve_event_type_model {
        return true;
    }

    let model = Model::from_ptr(event as *const clingo_sys::clingo_model_t);

    match (state.closure)(&model) {
        Ok(cont) => {
            unsafe { *goon = cont };
            true
        }
        Err(e) => {
            state.error = Some(e);
            unsafe { *goon = false };
            true
        }
    }
}

/// Solve the program, calling `on_model` for each model found.
///
/// Returns the final solve result. The callback receives a `&Model` and
/// returns `Ok(true)` to continue searching or `Ok(false)` to stop.
pub(crate) fn solve_with_callback(
    control_ptr: *mut clingo_sys::clingo_control_t,
    mut on_model: impl FnMut(&Model) -> Result<bool, Error>,
) -> Result<SolveResult, Error> {
    let mut state = CallbackState {
        closure: &mut on_model,
        error: None,
    };

    let mut handle: *mut clingo_sys::clingo_solve_handle_t = ptr::null_mut();
    check(unsafe {
        clingo_sys::clingo_control_solve(
            control_ptr,
            0, // synchronous, no yield
            ptr::null(),
            0,
            Some(solve_trampoline),
            &mut state as *mut CallbackState as *mut c_void,
            &mut handle,
        )
    })?;

    // Block until solving is done, then always close the handle.
    let mut result_bits: u32 = 0;
    let get_ok = unsafe { clingo_sys::clingo_solve_handle_get(handle, &mut result_bits) };
    let close_ok = unsafe { clingo_sys::clingo_solve_handle_close(handle) };

    // Callback errors take priority over clingo errors.
    if let Some(e) = state.error {
        return Err(e);
    }

    check(get_ok)?;
    check(close_ok)?;

    Ok(SolveResult::from_bitset(result_bits))
}
