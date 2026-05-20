use std::fs;

use aspire::{Control, ShowType, Symbol};

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

#[test]
fn load_file() {
    let path = std::env::temp_dir().join(format!(
        "aspire-control-load-{}-{}.lp",
        std::process::id(),
        "load_file"
    ));
    fs::write(&path, "a. b :- a.").unwrap();

    let mut ctl = Control::new(&[]).unwrap();
    ctl.load(path.to_str().unwrap()).unwrap();
    ctl.ground_base().unwrap();

    let mut handle = ctl.solve_iter().unwrap();
    let model = handle.next_model().unwrap().unwrap();
    let symbols = model.symbols(ShowType::Shown).unwrap();
    handle.close().unwrap();

    fs::remove_file(path).unwrap();

    assert!(symbols.contains(&Symbol::id("a", true).unwrap()));
    assert!(symbols.contains(&Symbol::id("b", true).unwrap()));
}
