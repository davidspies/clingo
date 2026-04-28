use crate::ShowType;
use crate::error::{ClingoError, check};
use crate::symbol::Symbol;

/// A model found during solving. Only valid for the duration of the callback.
pub(super) struct RawModel {
    ptr: *const clingo_sys::clingo_model_t,
}

impl RawModel {
    pub(super) unsafe fn from_ptr(ptr: *const clingo_sys::clingo_model_t) -> Self {
        RawModel { ptr }
    }

    /// Get the symbols in this model matching the given show type.
    pub(super) fn symbols(&self, show: ShowType) -> Result<Vec<Symbol>, ClingoError> {
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
    pub(super) fn contains(&self, atom: Symbol) -> Result<bool, ClingoError> {
        let mut contained = false;
        check(unsafe { clingo_sys::clingo_model_contains(self.ptr, atom.raw(), &mut contained) })?;
        Ok(contained)
    }

    /// Get the running number of this model.
    pub(super) fn number(&self) -> Result<u64, ClingoError> {
        let mut n: u64 = 0;
        check(unsafe { clingo_sys::clingo_model_number(self.ptr, &mut n) })?;
        Ok(n)
    }

    /// Whether the optimality of this model has been proven.
    pub(super) fn optimality_proven(&self) -> Result<bool, ClingoError> {
        let mut proven = false;
        check(unsafe { clingo_sys::clingo_model_optimality_proven(self.ptr, &mut proven) })?;
        Ok(proven)
    }

    /// Get the cost vector of this model.
    pub(super) fn cost(&self) -> Result<Vec<i64>, ClingoError> {
        let mut size: usize = 0;
        check(unsafe { clingo_sys::clingo_model_cost_size(self.ptr, &mut size) })?;
        let mut costs = vec![0i64; size];
        check(unsafe { clingo_sys::clingo_model_cost(self.ptr, costs.as_mut_ptr(), size) })?;
        Ok(costs)
    }
}
