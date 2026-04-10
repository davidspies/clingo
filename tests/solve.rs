use clingo::{Control, ShowType, Symbol};

#[test]
fn solve_simple() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "a. b :- a.").unwrap();
    ctl.ground_base().unwrap();

    let mut handle = ctl.solve_iter().unwrap();
    let mut models = Vec::new();
    while let Some(model) = handle.next_model().unwrap() {
        models.push(model.symbols(ShowType::Shown).unwrap());
    }
    let result = handle.close().unwrap();

    assert!(result.satisfiable);
    assert!(result.exhausted);
    assert_eq!(models.len(), 1);

    let a = Symbol::id("a", true).unwrap();
    let b = Symbol::id("b", true).unwrap();
    assert!(models[0].contains(&a));
    assert!(models[0].contains(&b));
}

#[test]
fn solve_multiple_models() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "{a}. {b}.").unwrap();
    ctl.ground_base().unwrap();

    let mut handle = ctl.solve_iter().unwrap();
    let mut count = 0;
    while handle.next_model().unwrap().is_some() {
        count += 1;
    }
    let result = handle.close().unwrap();

    assert!(result.satisfiable);
    assert!(result.exhausted);
    assert_eq!(count, 4);
}

#[test]
fn solve_stop_early() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "{a}. {b}. {c}.").unwrap();
    ctl.ground_base().unwrap();

    let mut handle = ctl.solve_iter().unwrap();
    let model = handle.next_model().unwrap();
    assert!(model.is_some());
    // drop handle without exhausting — stops early
    let result = handle.close().unwrap();
    assert!(result.satisfiable);
    assert!(!result.exhausted);
}

#[test]
fn solve_unsat() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "a. :- a.").unwrap();
    ctl.ground_base().unwrap();

    let mut handle = ctl.solve_iter().unwrap();
    assert!(handle.next_model().unwrap().is_none());
    let result = handle.close().unwrap();
    assert!(result.unsatisfiable);
}

#[test]
fn model_contains() {
    let mut ctl = Control::new(&[]).unwrap();
    ctl.add("base", &[], "a. b.").unwrap();
    ctl.ground_base().unwrap();

    let a = Symbol::id("a", true).unwrap();
    let c = Symbol::id("c", true).unwrap();

    let mut handle = ctl.solve_iter().unwrap();
    let model = handle.next_model().unwrap().unwrap();
    assert!(model.contains(a).unwrap());
    assert!(!model.contains(c).unwrap());
}

#[test]
fn model_number() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "{a}.").unwrap();
    ctl.ground_base().unwrap();

    let mut handle = ctl.solve_iter().unwrap();
    let mut numbers = Vec::new();
    while let Some(model) = handle.next_model().unwrap() {
        numbers.push(model.number().unwrap());
    }

    numbers.sort();
    assert_eq!(numbers, vec![1, 2]);
}
