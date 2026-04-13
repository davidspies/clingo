use ringo::{Control, ShowType, Symbol, TruthValue};

#[test]
fn assign_external_true() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "#external a. b :- a.").unwrap();
    ctl.ground_base().unwrap();

    // Default: external is false, so just one empty model
    let mut handle = ctl.solve_iter().unwrap();
    let mut count = 0;
    while handle.next_model().unwrap().is_some() {
        count += 1;
    }
    handle.close().unwrap();
    assert_eq!(count, 1);

    // Assign a=true
    let a = Symbol::id("a", true).unwrap();
    assert!(ctl.assign_external(a, TruthValue::True).unwrap());

    let mut handle = ctl.solve_iter().unwrap();
    let model = handle.next_model().unwrap().unwrap();
    let syms = model.symbols(ShowType::Shown).unwrap();
    assert!(syms.iter().any(|s| s.name() == Some("a")));
}

#[test]
fn assign_external_false() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "#external a. b :- a.").unwrap();
    ctl.ground_base().unwrap();

    let a = Symbol::id("a", true).unwrap();
    let b = Symbol::id("b", true).unwrap();

    // Set true, then back to false
    ctl.assign_external(a, TruthValue::True).unwrap();
    ctl.assign_external(a, TruthValue::False).unwrap();

    let mut handle = ctl.solve_iter().unwrap();
    let model = handle.next_model().unwrap().unwrap();
    assert!(!model.contains(a).unwrap());
    assert!(!model.contains(b).unwrap());
}

#[test]
fn assign_external_free() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "#external a.").unwrap();
    ctl.ground_base().unwrap();

    let a = Symbol::id("a", true).unwrap();
    ctl.assign_external(a, TruthValue::Free).unwrap();

    let mut handle = ctl.solve_iter().unwrap();
    let mut count = 0;
    while handle.next_model().unwrap().is_some() {
        count += 1;
    }
    handle.close().unwrap();
    assert_eq!(count, 2);
}

#[test]
fn release_external() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "#external a.").unwrap();
    ctl.ground_base().unwrap();

    let a = Symbol::id("a", true).unwrap();
    assert!(ctl.release_external(a).unwrap());

    let mut handle = ctl.solve_iter().unwrap();
    let model = handle.next_model().unwrap().unwrap();
    assert!(!model.contains(a).unwrap());
    assert!(handle.next_model().unwrap().is_none());
}

#[test]
fn assign_nonexistent_symbol() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "a.").unwrap();
    ctl.ground_base().unwrap();

    let z = Symbol::id("z", true).unwrap();
    assert!(!ctl.assign_external(z, TruthValue::True).unwrap());
}
