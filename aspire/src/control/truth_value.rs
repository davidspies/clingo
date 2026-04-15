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
    pub(crate) fn to_raw(self) -> clingo_sys::clingo_truth_value_t {
        match self {
            TruthValue::Free => clingo_sys::clingo_truth_value_e_clingo_truth_value_free as i32,
            TruthValue::True => clingo_sys::clingo_truth_value_e_clingo_truth_value_true as i32,
            TruthValue::False => clingo_sys::clingo_truth_value_e_clingo_truth_value_false as i32,
        }
    }
}
