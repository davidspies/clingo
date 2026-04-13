use aspire::{Symbol, Symbolic};

#[derive(Debug, PartialEq, Symbolic)]
struct MyPred(i32, i32);

#[derive(Debug, PartialEq, Symbolic)]
enum Cell {
    Coords(i32, i32),
    NoCell,
}

#[test]
fn struct_to_symbol() {
    let pred = MyPred(3, 16);
    let sym = pred.to_symbol();
    assert_eq!(sym.to_string_lossy().unwrap(), "my_pred(3,16)");
}

#[test]
fn struct_from_symbol() {
    let sym = Symbol::parse("my_pred(3,16)").unwrap();
    let pred = MyPred::from_symbol(sym).unwrap();
    assert_eq!(pred, MyPred(3, 16));
}

#[test]
fn struct_roundtrip() {
    let pred = MyPred(42, -7);
    let recovered = MyPred::from_symbol(pred.to_symbol()).unwrap();
    assert_eq!(recovered, pred);
}

#[test]
fn enum_tuple_variant() {
    let cell = Cell::Coords(2, 3);
    let sym = cell.to_symbol();
    assert_eq!(sym.to_string_lossy().unwrap(), "coords(2,3)");

    let recovered = Cell::from_symbol(sym).unwrap();
    assert_eq!(recovered, Cell::Coords(2, 3));
}

#[test]
fn enum_unit_variant() {
    let cell = Cell::NoCell;
    let sym = cell.to_symbol();
    assert_eq!(sym.to_string_lossy().unwrap(), "no_cell");

    let recovered = Cell::from_symbol(sym).unwrap();
    assert_eq!(recovered, Cell::NoCell);
}

#[test]
fn enum_from_parsed() {
    let sym = Symbol::parse("coords(2,3)").unwrap();
    assert_eq!(Cell::from_symbol(sym).unwrap(), Cell::Coords(2, 3));

    let sym = Symbol::parse("no_cell").unwrap();
    assert_eq!(Cell::from_symbol(sym).unwrap(), Cell::NoCell);
}

#[test]
fn wrong_name_returns_none() {
    let sym = Symbol::parse("other(1,2)").unwrap();
    assert!(MyPred::from_symbol(sym).is_none());
    assert!(Cell::from_symbol(sym).is_none());
}

#[test]
fn wrong_arity_returns_none() {
    let sym = Symbol::parse("my_pred(1)").unwrap();
    assert!(MyPred::from_symbol(sym).is_none());
}

#[derive(Debug, PartialEq, Symbolic)]
struct MyFn(i32, i32);

#[derive(Debug, PartialEq, Symbolic)]
struct MyPredNested(MyFn, i32);

#[test]
fn nested_struct_to_symbol() {
    let pred = MyPredNested(MyFn(1, 2), 16);
    let sym = pred.to_symbol();
    assert_eq!(
        sym.to_string_lossy().unwrap(),
        "my_pred_nested(my_fn(1,2),16)"
    );
}

#[test]
fn nested_struct_from_parsed() {
    let sym = Symbol::parse("my_pred_nested(my_fn(1,2),16)").unwrap();
    let pred = MyPredNested::from_symbol(sym).unwrap();
    assert_eq!(pred, MyPredNested(MyFn(1, 2), 16));
}

#[test]
fn number_returns_none() {
    let sym = Symbol::number(42);
    assert!(MyPred::from_symbol(sym).is_none());
    assert!(Cell::from_symbol(sym).is_none());
}
