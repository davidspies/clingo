use clingo::{Atom, Control, ExternalType, GroundStatement, Sign, Symbol};

#[test]
fn observe_rules() {
    let mut ctl = Control::new(&[]).unwrap();
    // Non-trivial program so atoms aren't simplified away
    ctl.add("base", &[], "a :- not b. b :- not a.").unwrap();
    let stmts = ctl.ground_base_observed().unwrap();

    let rules: Vec<_> = stmts
        .iter()
        .filter(|s| matches!(s, GroundStatement::Rule { .. }))
        .collect();
    assert!(!rules.is_empty());

    let a = Symbol::id("a", true).unwrap();
    let b = Symbol::id("b", true).unwrap();

    // "a :- not b." should appear with positive head a, negative body b
    let has_a_rule = stmts.iter().any(|s| match s {
        GroundStatement::Rule {
            choice: false,
            head,
            body,
        } => {
            head.iter().any(|at| *at == Atom::Symbol(a))
                && body
                    .iter()
                    .any(|lit| lit.0 == Sign::Neg && lit.1 == Atom::Symbol(b))
        }
        _ => false,
    });
    assert!(has_a_rule);
}

#[test]
fn observe_choice_rule() {
    let mut ctl = Control::new(&[]).unwrap();
    ctl.add("base", &[], "{a}. {b}.").unwrap();
    let stmts = ctl.ground_base_observed().unwrap();

    let choice_rules: Vec<_> = stmts
        .iter()
        .filter(|s| matches!(s, GroundStatement::Rule { choice: true, .. }))
        .collect();
    assert!(!choice_rules.is_empty());
}

#[test]
fn observe_external() {
    let mut ctl = Control::new(&[]).unwrap();
    ctl.add("base", &[], "#external a.").unwrap();
    let stmts = ctl.ground_base_observed().unwrap();

    let externals: Vec<_> = stmts
        .iter()
        .filter(|s| matches!(s, GroundStatement::External { .. }))
        .collect();
    assert!(!externals.is_empty());
    assert!(externals.iter().any(|s| matches!(
        s,
        GroundStatement::External {
            external_type: ExternalType::False,
            ..
        }
    )));
}

#[test]
fn still_solvable_after_observe() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "a :- not b. b :- not a.").unwrap();
    let _stmts = ctl.ground_base_observed().unwrap();

    let mut handle = ctl.solve_iter().unwrap();
    let model = handle.next_model().unwrap();
    assert!(model.is_some());
}

#[test]
fn facts_simplified_away() {
    // Pure facts are simplified by the grounder — they don't appear
    // as resolved symbols in the observer output.
    let mut ctl = Control::new(&[]).unwrap();
    ctl.add("base", &[], "a.").unwrap();
    let stmts = ctl.ground_base_observed().unwrap();

    // The rule exists but the head atom is Aux (no output_atom mapping)
    let rules: Vec<_> = stmts
        .iter()
        .filter(|s| matches!(s, GroundStatement::Rule { .. }))
        .collect();
    assert!(!rules.is_empty());
}
