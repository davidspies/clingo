use clingo::{Control, ShowType, Symbol, TruthValue};

#[test]
fn assign_external_true() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "#external a. b :- a.").unwrap();
    ctl.ground_base().unwrap();

    // With a=free, both {}, {a,b} are possible, but by default external is false
    let mut count = 0;
    ctl.solve(|_| {
        count += 1;
        Ok(true)
    })
    .unwrap();
    assert_eq!(count, 1); // just the empty model

    // Assign a=true
    let a = Symbol::id("a", true).unwrap();
    assert!(ctl.assign_external(a, TruthValue::True).unwrap());

    let mut found_a = false;
    ctl.solve(|model| {
        let syms = model.symbols(ShowType::Shown)?;
        found_a = syms.iter().any(|s| s.name() == Some("a"));
        Ok(true)
    })
    .unwrap();
    assert!(found_a);
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

    ctl.solve(|model| {
        assert!(!model.contains(a)?);
        assert!(!model.contains(b)?);
        Ok(true)
    })
    .unwrap();
}

#[test]
fn assign_external_free() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "#external a.").unwrap();
    ctl.ground_base().unwrap();

    let a = Symbol::id("a", true).unwrap();
    ctl.assign_external(a, TruthValue::Free).unwrap();

    // Free means solver decides — with {a} choice, we get 2 models
    let mut count = 0;
    ctl.solve(|_| {
        count += 1;
        Ok(true)
    })
    .unwrap();
    assert_eq!(count, 2);
}

#[test]
fn release_external() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "#external a.").unwrap();
    ctl.ground_base().unwrap();

    let a = Symbol::id("a", true).unwrap();
    assert!(ctl.release_external(a).unwrap());

    // After release, a is no longer external — it's just false
    let mut count = 0;
    ctl.solve(|model| {
        assert!(!model.contains(a)?);
        count += 1;
        Ok(true)
    })
    .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn assign_nonexistent_symbol() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "a.").unwrap();
    ctl.ground_base().unwrap();

    let z = Symbol::id("z", true).unwrap();
    assert!(!ctl.assign_external(z, TruthValue::True).unwrap());
}
