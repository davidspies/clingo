use clingo::{Control, Error, ShowType, Symbol};

#[test]
fn solve_simple() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "a. b :- a.").unwrap();
    ctl.ground_base().unwrap();

    let mut models = Vec::new();
    let result = ctl
        .solve(|model| {
            let syms = model.symbols(ShowType::Shown)?;
            models.push(syms);
            Ok(true)
        })
        .unwrap();

    assert!(result.satisfiable);
    assert!(result.exhausted);
    assert_eq!(models.len(), 1);

    let syms = &models[0];
    let a = Symbol::id("a", true).unwrap();
    let b = Symbol::id("b", true).unwrap();
    assert!(syms.contains(&a));
    assert!(syms.contains(&b));
}

#[test]
fn solve_multiple_models() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "{a}. {b}.").unwrap();
    ctl.ground_base().unwrap();

    let mut count = 0;
    let result = ctl
        .solve(|_model| {
            count += 1;
            Ok(true)
        })
        .unwrap();

    assert!(result.satisfiable);
    assert!(result.exhausted);
    // {a}. {b}. has 4 answer sets: {}, {a}, {b}, {a,b}
    assert_eq!(count, 4);
}

#[test]
fn solve_stop_early() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "{a}. {b}. {c}.").unwrap();
    ctl.ground_base().unwrap();

    let mut count = 0;
    let result = ctl
        .solve(|_model| {
            count += 1;
            Ok(false) // stop after first model
        })
        .unwrap();

    assert!(result.satisfiable);
    assert!(!result.exhausted);
    assert_eq!(count, 1);
}

#[test]
fn solve_unsat() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "a. :- a.").unwrap();
    ctl.ground_base().unwrap();

    let mut count = 0;
    let result = ctl
        .solve(|_model| {
            count += 1;
            Ok(true)
        })
        .unwrap();

    assert!(result.unsatisfiable);
    assert_eq!(count, 0);
}

#[test]
fn model_contains() {
    let mut ctl = Control::new(&[]).unwrap();
    ctl.add("base", &[], "a. b.").unwrap();
    ctl.ground_base().unwrap();

    let a = Symbol::id("a", true).unwrap();
    let c = Symbol::id("c", true).unwrap();

    ctl.solve(|model| {
        assert!(model.contains(a)?);
        assert!(!model.contains(c)?);
        Ok(true)
    })
    .unwrap();
}

#[test]
fn model_number() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add("base", &[], "{a}.").unwrap();
    ctl.ground_base().unwrap();

    let mut numbers = Vec::new();
    ctl.solve(|model| {
        numbers.push(model.number()?);
        Ok(true)
    })
    .unwrap();

    numbers.sort();
    assert_eq!(numbers, vec![1, 2]);
}

#[test]
fn callback_error_propagates() {
    let mut ctl = Control::new(&[]).unwrap();
    ctl.add("base", &[], "a.").unwrap();
    ctl.ground_base().unwrap();

    let result: Result<_, Error> = ctl.solve(|_model| {
        // CString::new with an interior NUL byte produces a NulError
        Err(std::ffi::CString::new("ab\0cd").unwrap_err().into())
    });

    assert!(result.is_err());
}
