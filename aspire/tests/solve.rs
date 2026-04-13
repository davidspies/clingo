use aspire::{Control, Error, ShowType, Symbol, Symbolic};

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

#[derive(Debug, PartialEq, Symbolic)]
struct Edge(i32, i32);

#[derive(Debug, PartialEq, Symbolic)]
struct Node(i32);

#[test]
fn model_atoms() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add(
        "base",
        &[],
        "edge(1,2). edge(2,3). node(1). node(2). node(3).",
    )
    .unwrap();
    ctl.ground_base().unwrap();

    let mut handle = ctl.solve_iter().unwrap();
    let model = handle.next_model().unwrap().unwrap();

    let edges: Vec<Edge> = model.atoms().unwrap();
    assert_eq!(edges.len(), 2);
    assert!(edges.contains(&Edge(1, 2)));
    assert!(edges.contains(&Edge(2, 3)));

    let nodes: Vec<Node> = model.atoms().unwrap();
    assert_eq!(nodes.len(), 3);
}

#[test]
fn model_atoms_no_matches() {
    let mut ctl = Control::new(&[]).unwrap();
    ctl.add("base", &[], "a. b.").unwrap();
    ctl.ground_base().unwrap();

    let mut handle = ctl.solve_iter().unwrap();
    let model = handle.next_model().unwrap().unwrap();

    let edges: Vec<Edge> = model.atoms().unwrap();
    assert!(edges.is_empty());
}

#[test]
fn model_atoms_with_choice() {
    let mut ctl = Control::new(&["0"]).unwrap();
    ctl.add(
        "base",
        &[],
        "node(1). node(2). {edge(X,Y)} :- node(X), node(Y).",
    )
    .unwrap();
    ctl.ground_base().unwrap();

    let mut handle = ctl.solve_iter().unwrap();
    while let Some(model) = handle.next_model().unwrap() {
        let edges: Vec<Edge> = model.atoms().unwrap();
        let nodes: Vec<Node> = model.atoms().unwrap();
        // Every model should have exactly 2 nodes
        assert_eq!(nodes.len(), 2);
        // Edges vary per model but should all parse correctly
        for Edge(a, b) in &edges {
            assert!(*a >= 1 && *a <= 2);
            assert!(*b >= 1 && *b <= 2);
        }
    }
    handle.close().unwrap();
}

#[test]
fn from_symbol_result_error() {
    // A symbol with wrong structure for the target type should give a TypeMismatch error
    let sym = Symbol::parse("edge(1,2,3)").unwrap(); // arity 3, but Edge expects 2
    let result = Edge::from_symbol_result(sym);
    assert!(result.is_err());
    match result.unwrap_err() {
        Error::TypeMismatch(msg) => {
            assert!(msg.contains("edge(1,2,3)"), "message was: {msg}");
        }
        other => panic!("expected TypeMismatch, got: {other:?}"),
    }
}
