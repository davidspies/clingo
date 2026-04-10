use clingo::{Control, ExternalType, GroundStatement, Symbol};

#[test]
fn observe_simple_rules() {
    let mut ctl = Control::new(&[]).unwrap();
    ctl.add("base", &[], "a. b :- a.").unwrap();
    let stmts = ctl.ground_base_observed().unwrap();

    // Should have rules and output atoms
    let rules: Vec<_> = stmts
        .iter()
        .filter(|s| matches!(s, GroundStatement::Rule { .. }))
        .collect();
    assert!(!rules.is_empty());

    let outputs: Vec<_> = stmts
        .iter()
        .filter(|s| matches!(s, GroundStatement::OutputAtom { .. }))
        .collect();
    assert!(!outputs.is_empty());
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
fn observe_output_atom_symbols() {
    let mut ctl = Control::new(&[]).unwrap();
    ctl.add("base", &[], "edge(1,2). edge(2,3).").unwrap();
    let stmts = ctl.ground_base_observed().unwrap();

    let mut output_syms: Vec<String> = stmts
        .iter()
        .filter_map(|s| match s {
            GroundStatement::OutputAtom { symbol, .. } => Some(symbol.to_string_lossy().unwrap()),
            _ => None,
        })
        .collect();
    output_syms.sort();
    assert_eq!(output_syms, vec!["edge(1,2)", "edge(2,3)"]);
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
fn observe_with_params() {
    let mut ctl = Control::new(&[]).unwrap();
    ctl.add("step", &["t"], "at(t).").unwrap();
    let t = Symbol::number(1);
    let stmts = ctl.ground_observed(&[("step", &[t])]).unwrap();

    let mut output_syms: Vec<String> = stmts
        .iter()
        .filter_map(|s| match s {
            GroundStatement::OutputAtom { symbol, .. } => Some(symbol.to_string_lossy().unwrap()),
            _ => None,
        })
        .collect();
    output_syms.sort();
    assert_eq!(output_syms, vec!["at(1)"]);
}

#[test]
fn still_solvable_after_observe() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "a. b :- a.").unwrap();
    let _stmts = ctl.ground_base_observed().unwrap();

    let mut handle = ctl.solve_iter().unwrap();
    let model = handle.next_model().unwrap();
    assert!(model.is_some());
}
