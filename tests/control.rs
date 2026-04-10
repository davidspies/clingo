use clingo::{Control, Symbol};

#[test]
fn create_and_drop() {
    let _ctl = Control::new(&[]).unwrap();
}

#[test]
fn add_and_ground() {
    let mut ctl = Control::new(&[]).unwrap();
    ctl.add("base", &[], "a. b :- a.").unwrap();
    ctl.ground_base().unwrap();
}

#[test]
fn bad_program() {
    let mut ctl = Control::new(&[]).unwrap();
    ctl.add("base", &[], "a. b :- a.").unwrap();
    let result = ctl.add("base", &[], "this is not valid asp !!!");
    assert!(result.is_err());
}

#[test]
fn ground_with_params() {
    let mut ctl = Control::new(&[]).unwrap();
    ctl.add("step", &["t"], "b(t).").unwrap();
    let t = Symbol::number(1);
    ctl.ground(&[("step", &[t])]).unwrap();
}
