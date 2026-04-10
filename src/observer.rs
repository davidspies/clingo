use std::os::raw::c_void;

use crate::symbol::Symbol;

/// A literal in the ground program: positive means the atom, negative means its negation.
pub type Literal = i32;
/// An atom ID in the ground program.
pub type Atom = u32;

/// A single statement in the ground program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroundStatement {
    /// A normal or choice rule.
    Rule {
        choice: bool,
        head: Vec<Atom>,
        body: Vec<Literal>,
    },
    /// A weight rule (used for aggregates).
    WeightRule {
        choice: bool,
        head: Vec<Atom>,
        lower_bound: i32,
        body: Vec<(Literal, i32)>,
    },
    /// A minimize statement.
    Minimize {
        priority: i32,
        literals: Vec<(Literal, i32)>,
    },
    /// An external atom declaration.
    External {
        atom: Atom,
        external_type: ExternalType,
    },
    /// Maps an atom ID to a symbol (from `#show` or implicit output).
    OutputAtom { symbol: Symbol, atom: Atom },
}

/// The type of an external atom declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalType {
    Free,
    True,
    False,
    Release,
}

impl ExternalType {
    fn from_raw(t: clingo_sys::clingo_external_type_t) -> Self {
        match t as u32 {
            clingo_sys::clingo_external_type_e_clingo_external_type_free => ExternalType::Free,
            clingo_sys::clingo_external_type_e_clingo_external_type_true => ExternalType::True,
            clingo_sys::clingo_external_type_e_clingo_external_type_false => ExternalType::False,
            clingo_sys::clingo_external_type_e_clingo_external_type_release => {
                ExternalType::Release
            }
            _ => ExternalType::Free,
        }
    }
}

/// Safe wrapper around `slice::from_raw_parts` that handles null pointers with zero length.
unsafe fn raw_slice<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    if len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}

/// State accumulated by the observer callbacks.
pub(crate) struct ObserverState {
    pub statements: Vec<GroundStatement>,
}

unsafe extern "C" fn on_rule(
    choice: bool,
    head: *const clingo_sys::clingo_atom_t,
    head_size: usize,
    body: *const clingo_sys::clingo_literal_t,
    body_size: usize,
    data: *mut c_void,
) -> bool {
    let state = unsafe { &mut *(data as *mut ObserverState) };
    let head = unsafe { raw_slice(head, head_size) }.to_vec();
    let body = unsafe { raw_slice(body, body_size) }.to_vec();
    state
        .statements
        .push(GroundStatement::Rule { choice, head, body });
    true
}

unsafe extern "C" fn on_weight_rule(
    choice: bool,
    head: *const clingo_sys::clingo_atom_t,
    head_size: usize,
    lower_bound: clingo_sys::clingo_weight_t,
    body: *const clingo_sys::clingo_weighted_literal_t,
    body_size: usize,
    data: *mut c_void,
) -> bool {
    let state = unsafe { &mut *(data as *mut ObserverState) };
    let head = unsafe { raw_slice(head, head_size) }.to_vec();
    let raw_body = unsafe { raw_slice(body, body_size) };
    let body = raw_body.iter().map(|wl| (wl.literal, wl.weight)).collect();
    state.statements.push(GroundStatement::WeightRule {
        choice,
        head,
        lower_bound,
        body,
    });
    true
}

unsafe extern "C" fn on_external(
    atom: clingo_sys::clingo_atom_t,
    type_: clingo_sys::clingo_external_type_t,
    data: *mut c_void,
) -> bool {
    let state = unsafe { &mut *(data as *mut ObserverState) };
    state.statements.push(GroundStatement::External {
        atom,
        external_type: ExternalType::from_raw(type_),
    });
    true
}

unsafe extern "C" fn on_output_atom(
    symbol: clingo_sys::clingo_symbol_t,
    atom: clingo_sys::clingo_atom_t,
    data: *mut c_void,
) -> bool {
    let state = unsafe { &mut *(data as *mut ObserverState) };
    state.statements.push(GroundStatement::OutputAtom {
        symbol: unsafe { Symbol::from_raw(symbol) },
        atom,
    });
    true
}

pub(crate) fn make_observer() -> clingo_sys::clingo_ground_program_observer_t {
    clingo_sys::clingo_ground_program_observer_t {
        init_program: None,
        begin_step: None,
        end_step: None,
        rule: Some(on_rule),
        weight_rule: Some(on_weight_rule),
        minimize: None,
        project: None,
        output_atom: Some(on_output_atom),
        output_term: None,
        external: Some(on_external),
        assume: None,
        heuristic: None,
        acyc_edge: None,
        theory_term_number: None,
        theory_term_string: None,
        theory_term_compound: None,
        theory_element: None,
        theory_atom: None,
        theory_atom_with_guard: None,
    }
}
