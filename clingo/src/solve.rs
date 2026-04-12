use std::ptr;

use crate::control::Control;
use crate::error::{ClingoError, check};
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

/// A handle for iterating over models one at a time (yield mode).
///
/// Borrows the `Control` mutably, preventing concurrent access.
/// Call [`next_model`](Self::next_model) to advance, then
/// [`close`](Self::close) to get the final result.
/// If dropped without calling `close`, the handle is closed automatically
/// (but the result is discarded).
pub struct SolveHandle<'a> {
    handle: *mut clingo_sys::clingo_solve_handle_t,
    model: Option<Model>,
    control: &'a mut Control,
}

impl Drop for SolveHandle<'_> {
    fn drop(&mut self) {
        unsafe {
            clingo_sys::clingo_solve_handle_close(self.handle);
        }
    }
}

impl<'a> SolveHandle<'a> {
    /// Get a shared reference to the underlying `Control`.
    pub fn control(&self) -> &Control {
        self.control
    }

    /// Advance to the next model.
    ///
    /// Returns `Some(&Model)` or `None` when there are no more models.
    /// The returned reference borrows this handle mutably. To inspect the
    /// model while also accessing `control()`, use [`current_model`](Self::current_model)
    /// after this call returns.
    pub fn next_model(&mut self) -> Result<Option<&Model>, ClingoError> {
        check(unsafe { clingo_sys::clingo_solve_handle_resume(self.handle) })?;

        let mut model_ptr: *const clingo_sys::clingo_model_t = ptr::null();
        check(unsafe { clingo_sys::clingo_solve_handle_model(self.handle, &mut model_ptr) })?;

        if model_ptr.is_null() {
            self.model = None;
            Ok(None)
        } else {
            self.model = Some(Model::from_ptr(model_ptr));
            Ok(self.model.as_ref())
        }
    }

    /// Get the current model, if any.
    ///
    /// Returns `Some(&Model)` after a successful [`next_model`](Self::next_model)
    /// call. Unlike `next_model`, this borrows immutably, so you can
    /// simultaneously access [`control()`](Self::control).
    pub fn current_model(&self) -> Option<&Model> {
        self.model.as_ref()
    }

    /// Close the handle and return the final solve result.
    ///
    /// This consumes the handle. If you just drop it, the result is discarded.
    pub fn close(self) -> Result<SolveResult, ClingoError> {
        let mut result_bits: u32 = 0;
        check(unsafe { clingo_sys::clingo_solve_handle_get(self.handle, &mut result_bits) })?;
        // Drop will call clingo_solve_handle_close
        Ok(SolveResult::from_bitset(result_bits))
    }
}

pub(crate) fn solve_yielding(control: &mut Control) -> Result<SolveHandle<'_>, ClingoError> {
    let mut handle: *mut clingo_sys::clingo_solve_handle_t = ptr::null_mut();
    check(unsafe {
        clingo_sys::clingo_control_solve(
            control.ptr.as_ptr(),
            clingo_sys::clingo_solve_mode_e_clingo_solve_mode_yield,
            ptr::null(),
            0,
            None,
            ptr::null_mut(),
            &mut handle,
        )
    })?;

    Ok(SolveHandle {
        handle,
        model: None,
        control,
    })
}
