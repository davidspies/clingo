use crate::symbol::Symbol;

/// A function symbol with a statically known arity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fun<Args>(pub &'static str, pub Args);

pub type F0 = Fun<()>;

pub fn f0(name: &'static str) -> F0 {
    Fun(name, ())
}

pub trait FromSymbol: Sized {
    fn from_symbol(sym: Symbol) -> Option<Self>;
}

pub trait FromSymbols: Sized {
    fn from_symbols(syms: Vec<Symbol>) -> Option<Self>;
}

impl FromSymbol for Symbol {
    fn from_symbol(sym: Symbol) -> Option<Self> {
        Some(sym)
    }
}

impl FromSymbol for i32 {
    fn from_symbol(sym: Symbol) -> Option<Self> {
        sym.as_number()
    }
}

impl FromSymbol for &'static str {
    fn from_symbol(sym: Symbol) -> Option<Self> {
        sym.as_string()
    }
}

impl<Args: FromSymbols> FromSymbol for Fun<Args> {
    fn from_symbol(sym: Symbol) -> Option<Self> {
        let name = sym.name()?;
        let args = sym.arguments()?;
        Some(Self(name, Args::from_symbols(args)?))
    }
}

impl<T: FromSymbol> FromSymbols for T {
    fn from_symbols(syms: Vec<Symbol>) -> Option<Self> {
        if syms.len() == 1 {
            Some(T::from_symbol(syms[0])?)
        } else {
            None
        }
    }
}

impl<T: FromSymbol, const N: usize> FromSymbols for [T; N] {
    fn from_symbols(syms: Vec<Symbol>) -> Option<Self> {
        if syms.len() == N {
            let ts: Option<Vec<T>> = syms.into_iter().map(T::from_symbol).collect();
            Some(
                ts?.try_into()
                    .unwrap_or_else(|_| unreachable!("Length already checked")),
            )
        } else {
            None
        }
    }
}

macro_rules! impl_from_symbols_for_tuple {
    ($($T:ident),*) => {
        impl<$($T: FromSymbol),*> FromSymbols for ($($T,)*) {
            fn from_symbols(syms: Vec<Symbol>) -> Option<Self> {
                let mut iter = syms.into_iter();
                let result = ($($T::from_symbol(iter.next()?)?,)*);
                if iter.next().is_some() { return None; }
                Some(result)
            }
        }
    };
}

impl_from_symbols_for_tuple!();
impl_from_symbols_for_tuple!(A);
impl_from_symbols_for_tuple!(A, B);
impl_from_symbols_for_tuple!(A, B, C);
impl_from_symbols_for_tuple!(A, B, C, D);
impl_from_symbols_for_tuple!(A, B, C, D, E);
impl_from_symbols_for_tuple!(A, B, C, D, E, F);
impl_from_symbols_for_tuple!(A, B, C, D, E, F, G);
impl_from_symbols_for_tuple!(A, B, C, D, E, F, G, H);
impl_from_symbols_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_from_symbols_for_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_from_symbols_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_from_symbols_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
