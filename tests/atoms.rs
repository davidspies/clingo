use clingo::{Control, F0, Fun, Symbolic};

#[test]
fn iterate_atoms_by_signature() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add(
        "base",
        &[],
        "edge(1,2). edge(2,3). node(1). node(2). node(3).",
    )
    .unwrap();
    ctl.ground_base().unwrap();

    let edges: Vec<Fun<(i32, i32)>> = ctl.atoms("edge").unwrap();
    assert_eq!(edges.len(), 2);
    assert!(edges.contains(&Fun("edge", (1, 2))));
    assert!(edges.contains(&Fun("edge", (2, 3))));

    let nodes: Vec<Fun<(i32,)>> = ctl.atoms("node").unwrap();
    assert_eq!(nodes.len(), 3);
}

#[test]
fn atoms_during_solve() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add(
        "base",
        &[],
        "edge(1,2). edge(2,3). {path(X,Y)} :- edge(X,Y).",
    )
    .unwrap();
    ctl.ground_base().unwrap();

    let mut handle = ctl.solve_iter().unwrap();

    while handle.next_model().unwrap().is_some() {
        let model = handle.current_model().unwrap();
        let edges = handle.control().atoms::<(i32, i32)>("edge").unwrap();
        for &Fun(name, (a, b)) in &edges {
            assert_eq!(name, "edge");
            let sym = Fun("path", (a, b)).to_symbol();
            let _in_model = model.contains(sym).unwrap();
        }
    }
    handle.close().unwrap();
}

#[test]
fn atoms_with_symbol_args() {
    let mut ctl = Control::new(&[]).unwrap();
    ctl.add("base", &[], "f(a,1). f(b,2). f(c,3).").unwrap();
    ctl.ground_base().unwrap();

    let funs: Vec<Fun<(F0, i32)>> = ctl.atoms("f").unwrap();
    assert_eq!(funs.len(), 3);
}

#[test]
fn atoms_no_matches() {
    let mut ctl = Control::new(&[]).unwrap();
    ctl.add("base", &[], "a. b.").unwrap();
    ctl.ground_base().unwrap();

    let result: Vec<Fun<i32>> = ctl.atoms("nonexistent").unwrap();
    assert!(result.is_empty());
}
