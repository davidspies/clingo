use clingo::{F0, Fun, Symbol, SymbolType, SymbolValue, f0};
use std::collections::HashSet;

#[test]
fn number() {
    let s = Symbol::number(42);
    assert_eq!(s.symbol_type(), SymbolType::Number);
    assert_eq!(s.as_number(), Some(42));
    assert_eq!(s.name(), None);
    assert_eq!(s.to_string_lossy().unwrap(), "42");
}

#[test]
fn id() {
    let s = Symbol::id("abc", true).unwrap();
    assert_eq!(s.symbol_type(), SymbolType::Function);
    assert_eq!(s.name(), Some("abc"));
    assert_eq!(s.arguments(), Some(vec![]));
    assert_eq!(s.is_positive(), Some(true));
    assert_eq!(s.to_string_lossy().unwrap(), "abc");
}

#[test]
fn function() {
    let a = Symbol::id("a", true).unwrap();
    let b = Symbol::number(1);
    let f = Symbol::function("edge", &[a, b], true).unwrap();
    assert_eq!(f.symbol_type(), SymbolType::Function);
    assert_eq!(f.name(), Some("edge"));
    assert_eq!(f.arguments(), Some(vec![a, b]));
    assert_eq!(f.to_string_lossy().unwrap(), "edge(a,1)");
}

#[test]
fn negated() {
    let s = Symbol::id("foo", false).unwrap();
    assert_eq!(s.is_positive(), Some(false));
    assert_eq!(s.to_string_lossy().unwrap(), "-foo");
}

#[test]
fn string() {
    let s = Symbol::string("hello world").unwrap();
    assert_eq!(s.symbol_type(), SymbolType::String);
    assert_eq!(s.as_string(), Some("hello world"));
    assert_eq!(s.to_string_lossy().unwrap(), "\"hello world\"");
}

#[test]
fn special() {
    let inf = Symbol::infimum();
    let sup = Symbol::supremum();
    assert_eq!(inf.symbol_type(), SymbolType::Infimum);
    assert_eq!(sup.symbol_type(), SymbolType::Supremum);
    assert!(inf < sup);
}

#[test]
fn eq_hash() {
    let a1 = Symbol::id("a", true).unwrap();
    let a2 = Symbol::id("a", true).unwrap();
    let b = Symbol::id("b", true).unwrap();
    assert_eq!(a1, a2);
    assert_ne!(a1, b);

    let mut set = HashSet::new();
    set.insert(a1);
    set.insert(a2);
    set.insert(b);
    assert_eq!(set.len(), 2);
}

#[test]
fn value() {
    assert_eq!(Symbol::infimum().value(), SymbolValue::Infimum);
    assert_eq!(Symbol::supremum().value(), SymbolValue::Supremum);
    assert_eq!(Symbol::number(7).value(), SymbolValue::Number(7));
    assert_eq!(
        Symbol::string("hi").unwrap().value(),
        SymbolValue::String("hi"),
    );

    let a = Symbol::id("a", true).unwrap();
    let f = Symbol::function("edge", &[a, Symbol::number(1)], true).unwrap();
    assert_eq!(
        f.value(),
        SymbolValue::Function {
            name: "edge",
            arguments: vec![a, Symbol::number(1)],
            positive: true,
        },
    );

    let neg = Symbol::id("foo", false).unwrap();
    assert_eq!(
        neg.value(),
        SymbolValue::Function {
            name: "foo",
            arguments: vec![],
            positive: false,
        },
    );
}

#[test]
fn as_fun() {
    let a = Symbol::id("a", true).unwrap();
    let b = Symbol::number(1);
    let f = Symbol::function("edge", &[a, b], true).unwrap();

    let fixed = f.as_fun::<(F0, i32)>().unwrap();
    assert_eq!(fixed, Fun("edge", (f0("a"), 1)));
    let fixed = f.as_fun::<[Symbol; 2]>().unwrap();
    assert_eq!(fixed.0, "edge");
    assert_eq!(fixed.1, [a, b]);

    // wrong arity
    assert!(f.as_fun::<[Symbol; 0]>().is_none());
    assert!(f.as_fun::<[Symbol; 1]>().is_none());
    assert!(f.as_fun::<[Symbol; 3]>().is_none());

    // not a function
    assert!(Symbol::number(1).as_fun::<[Symbol; 0]>().is_none());

    // negative
    let neg = Symbol::id("foo", false).unwrap();
    assert!(neg.as_fun::<[Symbol; 0]>().is_none());

    // zero-arity positive id
    let id = Symbol::id("c", true).unwrap();
    let fixed: Fun<[Symbol; 0]> = id.as_fun().unwrap();
    assert_eq!(fixed.0, "c");
    assert_eq!(fixed.1, []);
}

#[test]
fn parse() {
    let s = Symbol::parse("edge(1,2)").unwrap();
    assert_eq!(
        s,
        Symbol::function("edge", &[Symbol::number(1), Symbol::number(2)], true).unwrap()
    );

    assert_eq!(Symbol::parse("42").unwrap(), Symbol::number(42));
    assert_eq!(Symbol::parse("a").unwrap(), Symbol::id("a", true).unwrap());
    assert_eq!(
        Symbol::parse("-a").unwrap(),
        Symbol::id("a", false).unwrap()
    );
    assert_eq!(
        Symbol::parse("\"hello\"").unwrap(),
        Symbol::string("hello").unwrap()
    );

    assert!(Symbol::parse("not valid!!!").is_err());
}
