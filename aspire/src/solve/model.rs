use std::{cell::OnceCell, collections::HashMap};

use crate::{ClingoError, Error, SymbolicFun, symbol::Symbol};

use super::{ShowType, raw_model::RawModel};

pub struct Model {
    cache: OnceCell<HashMap<(&'static str, usize), Vec<Symbol>>>,
    raw: RawModel,
}
impl Model {
    pub(super) fn new(raw: RawModel) -> Self {
        Self {
            cache: OnceCell::new(),
            raw,
        }
    }

    pub fn contains(&self, sym: Symbol) -> Result<bool, ClingoError> {
        self.raw.contains(sym)
    }

    pub fn symbols(&self, shown: ShowType) -> Result<Vec<Symbol>, ClingoError> {
        self.raw.symbols(shown)
    }

    pub fn number(&self) -> Result<u64, ClingoError> {
        self.raw.number()
    }

    pub fn optimality_proven(&self) -> Result<bool, ClingoError> {
        self.raw.optimality_proven()
    }

    /// Get the cost vector of this model.
    pub fn cost(&self) -> Result<Vec<i64>, ClingoError> {
        self.raw.cost()
    }

    pub fn atoms<T: SymbolicFun>(&self) -> Result<Vec<T>, Error> {
        if self.cache.get().is_none() {
            self.cache
                .set(build_index(self.symbols(ShowType::All)?))
                .unwrap();
        }
        let cache = self.cache.get().unwrap();
        let signature = T::signature();
        let Some(atoms) = cache.get(&signature) else {
            return Ok(vec![]);
        };
        atoms
            .iter()
            .map(|&sym| T::from_symbol_result(sym))
            .collect()
    }
}

#[track_caller]
fn build_index(symbols: Vec<Symbol>) -> HashMap<(&'static str, usize), Vec<Symbol>> {
    let mut map: HashMap<(&'static str, usize), Vec<Symbol>> = HashMap::new();
    for sym in symbols {
        let key = (sym.name().unwrap(), sym.arity().unwrap());
        map.entry(key).or_default().push(sym)
    }
    map
}
