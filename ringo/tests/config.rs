use ringo::Control;

#[test]
fn read_config() {
    let mut ctl = Control::new(&[]).unwrap();
    let cfg = ctl.configuration().unwrap();
    let val = cfg.get("solve.models").unwrap();
    // -1 means "compute all models" when no command-line arg is given
    assert_eq!(val.as_deref(), Some("-1"));
}

#[test]
fn write_config() {
    let mut ctl = Control::new(&[]).unwrap();
    let mut cfg = ctl.configuration().unwrap();
    cfg.set("solve.models", "0").unwrap();
    let val = cfg.get("solve.models").unwrap();
    assert_eq!(val.as_deref(), Some("0"));
}

#[test]
fn config_affects_solve() {
    let mut ctl = Control::new(&[]).unwrap();
    ctl.add("base", &[], "{a}. {b}.").unwrap();
    ctl.ground_base().unwrap();

    // Set models=0 via configuration instead of command-line arg
    let mut cfg = ctl.configuration().unwrap();
    cfg.set("solve.models", "0").unwrap();
    drop(cfg);

    let mut handle = ctl.solve_iter().unwrap();
    let mut count = 0;
    while handle.next_model().unwrap().is_some() {
        count += 1;
    }
    handle.close().unwrap();
    assert_eq!(count, 4);
}

#[test]
fn config_description() {
    let mut ctl = Control::new(&[]).unwrap();
    let cfg = ctl.configuration().unwrap();
    let solve = cfg.map_at("solve").unwrap();
    let desc = solve.description().unwrap();
    assert!(!desc.is_empty());
}

#[test]
fn config_array_access() {
    let mut ctl = Control::new(&[]).unwrap();
    let cfg = ctl.configuration().unwrap();
    let solver0 = cfg.map_at("solver").unwrap().array_at(0).unwrap();
    let desc = solver0.description().unwrap();
    assert!(!desc.is_empty());
}
