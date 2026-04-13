use aspire::{Control, ShowType, Symbol};

#[test]
fn iterate_models() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "{a}. {b}.").unwrap();
    ctl.ground_base().unwrap();

    let mut handle = ctl.solve_iter().unwrap();
    let mut count = 0;
    while let Some(_model) = handle.next_model().unwrap() {
        count += 1;
    }
    let result = handle.close().unwrap();

    assert_eq!(count, 4);
    assert!(result.satisfiable);
    assert!(result.exhausted);
}

#[test]
fn inspect_model() {
    let mut ctl = Control::new(&[]).unwrap();
    ctl.add("base", &[], "a. b :- a.").unwrap();
    ctl.ground_base().unwrap();

    let mut handle = ctl.solve_iter().unwrap();
    let model = handle.next_model().unwrap().unwrap();

    let syms = model.symbols(ShowType::Shown).unwrap();
    let a = Symbol::id("a", true).unwrap();
    let b = Symbol::id("b", true).unwrap();
    assert!(syms.contains(&a));
    assert!(syms.contains(&b));

    assert!(handle.next_model().unwrap().is_none());
    handle.close().unwrap();
}

#[test]
fn drop_without_close() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "{a}.").unwrap();
    ctl.ground_base().unwrap();

    {
        let mut handle = ctl.solve_iter().unwrap();
        let _ = handle.next_model().unwrap();
        // drop handle without close — should not leak or crash
    }

    // Control is usable again
    let mut handle = ctl.solve_iter().unwrap();
    let mut count = 0;
    while handle.next_model().unwrap().is_some() {
        count += 1;
    }
    handle.close().unwrap();
    assert_eq!(count, 2);
}

#[test]
fn unsat_iter() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "a. :- a.").unwrap();
    ctl.ground_base().unwrap();

    let mut handle = ctl.solve_iter().unwrap();
    assert!(handle.next_model().unwrap().is_none());
    let result = handle.close().unwrap();
    assert!(result.unsatisfiable);
}
