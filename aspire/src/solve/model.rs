use std::cell::OnceCell;

use crate::{ClingoError, Error, SymbolicFun, solve::ModelCache, symbol::Symbol};

use super::{ShowType, raw_model::RawModel};

pub struct Model {
    cache: OnceCell<ModelCache>,
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
        self.cache()?.atoms()
    }

    /// Get a borrowed reference to this model's [`ModelCache`], building it on
    /// first access.
    ///
    /// The cache is populated lazily from all shown symbols. To obtain an
    /// *owned* cache that outlives the solver, use
    /// [`SolveHandle::take_model_cache`](super::SolveHandle::take_model_cache).
    pub fn cache(&self) -> Result<&ModelCache, Error> {
        self.build_cache()?;
        Ok(self.cache.get().unwrap())
    }

    pub(super) fn take_cache(&mut self) -> Result<ModelCache, Error> {
        self.build_cache()?;
        Ok(self.cache.take().unwrap())
    }

    fn build_cache(&self) -> Result<(), Error> {
        if self.cache.get().is_none() {
            self.cache
                .set(ModelCache::from_symbols(self.symbols(ShowType::All)?))
                .unwrap();
        }
        Ok(())
    }
}
