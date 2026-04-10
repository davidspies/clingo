use clingo::{Control, F0, Fun, Symbol, Symbolic};

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

    let edges: Vec<(i32, i32)> = ctl.atoms("edge").unwrap();
    assert_eq!(edges.len(), 2);
    assert!(edges.contains(&(1, 2)));
    assert!(edges.contains(&(2, 3)));

    let nodes: Vec<i32> = ctl.atoms("node").unwrap();
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
        let edges: Vec<(i32, i32)> = handle.control().atoms("edge").unwrap();
        for &(a, b) in &edges {
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

    let funs: Vec<(F0, i32)> = ctl.atoms("f").unwrap();
    assert_eq!(funs.len(), 3);
}

#[test]
fn atoms_no_matches() {
    let mut ctl = Control::new(&[]).unwrap();
    ctl.add("base", &[], "a. b.").unwrap();
    ctl.ground_base().unwrap();

    let result: Vec<i32> = ctl.atoms("nonexistent").unwrap();
    assert!(result.is_empty());
}

#[test]
fn is_fact() {
    let mut ctl = Control::new(&[]).unwrap();
    ctl.add("base", &[], "a. b :- a. {c}.").unwrap();
    ctl.ground_base().unwrap();

    let a = Symbol::id("a", true).unwrap();
    let b = Symbol::id("b", true).unwrap();
    let c = Symbol::id("c", true).unwrap();
    let z = Symbol::id("z", true).unwrap();

    // a is a fact (unconditional)
    assert_eq!(ctl.is_fact(a).unwrap(), Some(true));
    // b is derived from a, which is a fact — so b is also a fact
    assert_eq!(ctl.is_fact(b).unwrap(), Some(true));
    // c is a choice — not a fact
    assert_eq!(ctl.is_fact(c).unwrap(), Some(false));
    // z doesn't exist
    assert_eq!(ctl.is_fact(z).unwrap(), None);
}
