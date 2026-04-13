use crate::symbol::Symbol;

/// A function symbol with a statically known arity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fun<Args>(pub &'static str, pub Args);

pub type F0 = Fun<()>;

pub fn f0(name: &'static str) -> F0 {
    Fun(name, ())
}

pub trait Symbolic: Sized {
    fn from_symbol(sym: Symbol) -> Option<Self>;
    fn to_symbol(&self) -> Symbol;
}

pub trait SymbolicFun: Symbolic {
    fn signature() -> (&'static str, usize);
}

pub trait SymbolicArgs: Sized {
    fn from_symbols(syms: Vec<Symbol>) -> Option<Self>;
    fn to_symbols(&self) -> Vec<Symbol>;
}

impl Symbolic for Symbol {
    fn from_symbol(sym: Symbol) -> Option<Self> {
        Some(sym)
    }
    fn to_symbol(&self) -> Symbol {
        *self
    }
}

impl Symbolic for i32 {
    fn from_symbol(sym: Symbol) -> Option<Self> {
        sym.as_number()
    }
    fn to_symbol(&self) -> Symbol {
        Symbol::number(*self)
    }
}

impl Symbolic for &'static str {
    fn from_symbol(sym: Symbol) -> Option<Self> {
        sym.as_string()
    }
    fn to_symbol(&self) -> Symbol {
        Symbol::string(self).unwrap()
    }
}

impl<Args: SymbolicArgs> Symbolic for Fun<Args> {
    fn from_symbol(sym: Symbol) -> Option<Self> {
        let name = sym.name()?;
        let args = sym.arguments()?;
        Some(Self(name, Args::from_symbols(args)?))
    }
    fn to_symbol(&self) -> Symbol {
        let &Self(name, ref args) = self;
        Symbol::function(name, &args.to_symbols(), true).unwrap()
    }
}

impl<T: Symbolic> SymbolicArgs for T {
    fn from_symbols(syms: Vec<Symbol>) -> Option<Self> {
        if syms.len() == 1 {
            Some(T::from_symbol(syms[0])?)
        } else {
            None
        }
    }
    fn to_symbols(&self) -> Vec<Symbol> {
        vec![self.to_symbol()]
    }
}

impl<T: Symbolic, const N: usize> SymbolicArgs for [T; N] {
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
    fn to_symbols(&self) -> Vec<Symbol> {
        self.iter().map(Symbolic::to_symbol).collect()
    }
}

macro_rules! impl_from_symbols_for_tuple {
    ($($T:ident),*) => {
        impl<$($T: Symbolic),*> SymbolicArgs for ($($T,)*) {
            fn from_symbols(syms: Vec<Symbol>) -> Option<Self> {
                let mut iter = syms.into_iter();
                let result = ($($T::from_symbol(iter.next()?)?,)*);
                if iter.next().is_some() { return None; }
                Some(result)
            }
            #[allow(non_snake_case)]
            fn to_symbols(&self) -> Vec<Symbol> {
                let ($($T,)*) = self;
                vec![$($T.to_symbol(),)*]
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
