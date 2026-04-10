use std::cell::RefCell;
use std::collections::HashMap;
use std::os::raw::c_void;

use crate::symbol::Symbol;

/// The type passed to clingo as the observer's data pointer.
pub(crate) type ObserverCell = RefCell<Option<ObserverState>>;

/// A single statement in the ground program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroundStatement<At = Atom, Lit = Literal> {
    /// A normal or choice rule.
    Rule {
        choice: bool,
        head: Vec<At>,
        body: Vec<Lit>,
    },
    /// A weight rule (used for aggregates).
    WeightRule {
        choice: bool,
        head: Vec<At>,
        lower_bound: i32,
        body: Vec<(Lit, i32)>,
    },
    /// A minimize statement.
    Minimize {
        priority: i32,
        literals: Vec<(Lit, i32)>,
    },
    /// An external atom declaration.
    External {
        atom: At,
        external_type: ExternalType,
    },
}

type GroundStatementInternal = GroundStatement<u32, i32>;

/// The sign of a literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sign {
    Pos,
    Neg,
}

/// A ground atom, either a user-visible symbol or an auxiliary grounder atom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Atom {
    Symbol(Symbol),
    Aux(u32),
}

/// A signed ground atom.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Literal(pub Sign, pub Atom);

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

// -- Internal state and callbacks --

pub(crate) struct ObserverState {
    statements: Vec<GroundStatementInternal>,
    atom_map: HashMap<u32, Symbol>,
}

impl ObserverState {
    pub fn new() -> Self {
        ObserverState {
            statements: Vec::new(),
            atom_map: HashMap::new(),
        }
    }

    pub fn into_statements(self) -> Vec<GroundStatement> {
        let ObserverState {
            statements,
            atom_map,
        } = self;

        let resolve_atom = |raw: u32| -> Atom {
            match atom_map.get(&raw) {
                Some(&sym) => Atom::Symbol(sym),
                None => Atom::Aux(raw),
            }
        };
        let resolve_literal = |raw: i32| -> Literal {
            if raw >= 0 {
                Literal(Sign::Pos, resolve_atom(raw as u32))
            } else {
                Literal(Sign::Neg, resolve_atom((-raw) as u32))
            }
        };

        statements
            .into_iter()
            .map(|stmt| match stmt {
                GroundStatementInternal::Rule { choice, head, body } => GroundStatement::Rule {
                    choice,
                    head: head.into_iter().map(&resolve_atom).collect(),
                    body: body.into_iter().map(&resolve_literal).collect(),
                },
                GroundStatementInternal::WeightRule {
                    choice,
                    head,
                    lower_bound,
                    body,
                } => GroundStatement::WeightRule {
                    choice,
                    head: head.into_iter().map(&resolve_atom).collect(),
                    lower_bound,
                    body: body
                        .into_iter()
                        .map(|(l, w)| (resolve_literal(l), w))
                        .collect(),
                },
                GroundStatementInternal::Minimize { priority, literals } => {
                    GroundStatement::Minimize {
                        priority,
                        literals: literals
                            .into_iter()
                            .map(|(l, w)| (resolve_literal(l), w))
                            .collect(),
                    }
                }
                GroundStatementInternal::External {
                    atom,
                    external_type,
                } => GroundStatement::External {
                    atom: resolve_atom(atom),
                    external_type,
                },
            })
            .collect()
    }
}

/// Borrow the observer cell; if it contains Some(state), run the closure.
fn with_state(data: *mut c_void, f: impl FnOnce(&mut ObserverState)) {
    let cell = unsafe { &*(data as *const ObserverCell) };
    if let Some(state) = cell.borrow_mut().as_mut() {
        f(state);
    }
}

unsafe extern "C" fn on_rule(
    choice: bool,
    head: *const clingo_sys::clingo_atom_t,
    head_size: usize,
    body: *const clingo_sys::clingo_literal_t,
    body_size: usize,
    data: *mut c_void,
) -> bool {
    with_state(data, |state| {
        let head = unsafe { raw_slice(head, head_size) }.to_vec();
        let body = unsafe { raw_slice(body, body_size) }.to_vec();
        state
            .statements
            .push(GroundStatementInternal::Rule { choice, head, body });
    });
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
    with_state(data, |state| {
        let head = unsafe { raw_slice(head, head_size) }.to_vec();
        let raw_body = unsafe { raw_slice(body, body_size) };
        let body = raw_body.iter().map(|wl| (wl.literal, wl.weight)).collect();
        state.statements.push(GroundStatementInternal::WeightRule {
            choice,
            head,
            lower_bound,
            body,
        });
    });
    true
}

unsafe extern "C" fn on_minimize(
    priority: clingo_sys::clingo_weight_t,
    literals: *const clingo_sys::clingo_weighted_literal_t,
    size: usize,
    data: *mut c_void,
) -> bool {
    with_state(data, |state| {
        let raw = unsafe { raw_slice(literals, size) };
        let literals = raw.iter().map(|wl| (wl.literal, wl.weight)).collect();
        state
            .statements
            .push(GroundStatementInternal::Minimize { priority, literals });
    });
    true
}

unsafe extern "C" fn on_external(
    atom: clingo_sys::clingo_atom_t,
    type_: clingo_sys::clingo_external_type_t,
    data: *mut c_void,
) -> bool {
    with_state(data, |state| {
        state.statements.push(GroundStatementInternal::External {
            atom,
            external_type: ExternalType::from_raw(type_),
        });
    });
    true
}

unsafe extern "C" fn on_output_atom(
    symbol: clingo_sys::clingo_symbol_t,
    atom: clingo_sys::clingo_atom_t,
    data: *mut c_void,
) -> bool {
    with_state(data, |state| {
        state
            .atom_map
            .insert(atom, unsafe { Symbol::from_raw(symbol) });
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
        minimize: Some(on_minimize),
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
