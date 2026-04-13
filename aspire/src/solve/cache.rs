use std::collections::HashMap;

use crate::{Symbol, SymbolicFun};

/// An owned, thread-safe snapshot of the atoms in a model.
///
/// Unlike [`Model`](super::Model), which borrows into the solver and is
/// invalidated on the next call to
/// [`SolveHandle::next_model`](super::SolveHandle::next_model), a
/// `ModelCache` is `Clone + Send + Sync` and can be freely cloned, moved to
/// other threads, or retained after the solver moves on to subsequent models.
///
/// Obtain one via [`Model::cache`](super::Model::cache) (borrowed) or
/// [`SolveHandle::take_model_cache`](super::SolveHandle::take_model_cache)
/// (owned).
#[derive(Clone, Debug)]
pub struct ModelCache(HashMap<(&'static str, usize), Vec<Symbol>>);

impl ModelCache {
    #[track_caller]
    pub(super) fn from_symbols(symbols: Vec<Symbol>) -> Self {
        let mut map: HashMap<(&'static str, usize), Vec<Symbol>> = HashMap::new();
        for sym in symbols {
            let key = (sym.name().unwrap(), sym.arity().unwrap());
            map.entry(key).or_default().push(sym)
        }
        Self(map)
    }

    /// Decode all cached atoms whose predicate signature matches `T`.
    ///
    /// Returns an empty `Vec` if no atoms with that signature are present.
    pub fn atoms<T: SymbolicFun>(&self) -> Result<Vec<T>, crate::Error> {
        let signature = T::signature();
        let Some(atoms) = self.0.get(&signature) else {
            return Ok(vec![]);
        };
        atoms
            .iter()
            .map(|&sym| T::from_symbol_result(sym))
            .collect()
    }

    /// Look up cached atoms by predicate name and arity, returning a slice of
    /// raw [`Symbol`] values.
    pub fn get_pred<'a, 'b: 'a>(&'a self, name: &'b str, arity: usize) -> &'a [Symbol] {
        match self.0.get(&(name, arity)) {
            None => &[],
            Some(atoms) => atoms.as_slice(),
        }
    }

    /// Iterate over every cached symbol, across all predicates.
    pub fn symbols(&self) -> impl Iterator<Item = Symbol> {
        self.0.values().flatten().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Static assertion: ModelCache must be Clone + Send + Sync.
    fn _assert_bounds<T: Clone + Send + Sync>() {}
    const _: fn() = _assert_bounds::<ModelCache>;

    fn make_cache(syms: &[&str]) -> ModelCache {
        let symbols: Vec<Symbol> = syms
            .iter()
            .map(|s| Symbol::parse(s).unwrap())
            .collect();
        ModelCache::from_symbols(symbols)
    }

    #[test]
    fn empty_cache() {
        let cache = ModelCache::from_symbols(vec![]);
        assert_eq!(cache.symbols().count(), 0);
        assert!(cache.get_pred("anything", 0).is_empty());
    }

    #[test]
    fn get_pred_groups_by_name_and_arity() {
        let cache = make_cache(&["edge(1,2)", "edge(3,4)", "node(1)"]);

        assert_eq!(cache.get_pred("edge", 2).len(), 2);
        assert_eq!(cache.get_pred("node", 1).len(), 1);
        assert!(cache.get_pred("edge", 1).is_empty());
        assert!(cache.get_pred("missing", 0).is_empty());
    }

    #[test]
    fn symbols_returns_all() {
        let cache = make_cache(&["a(1)", "b(2)", "a(3)"]);
        assert_eq!(cache.symbols().count(), 3);
    }

    #[test]
    fn clone_is_independent() {
        let cache = make_cache(&["foo(1)", "foo(2)"]);
        let clone = cache.clone();

        assert_eq!(cache.get_pred("foo", 1).len(), 2);
        assert_eq!(clone.get_pred("foo", 1).len(), 2);
    }

    #[test]
    fn send_to_another_thread() {
        let cache = make_cache(&["x(1)", "y(2,3)"]);
        let handle = std::thread::spawn(move || {
            assert_eq!(cache.get_pred("x", 1).len(), 1);
            assert_eq!(cache.get_pred("y", 2).len(), 1);
            cache.symbols().count()
        });
        assert_eq!(handle.join().unwrap(), 2);
    }

    #[test]
    fn share_across_threads() {
        let cache = std::sync::Arc::new(make_cache(&["p(1)", "p(2)", "q(0)"]));
        let threads: Vec<_> = (0..4)
            .map(|_| {
                let c = cache.clone();
                std::thread::spawn(move || c.get_pred("p", 1).len())
            })
            .collect();
        for t in threads {
            assert_eq!(t.join().unwrap(), 2);
        }
    }
}
