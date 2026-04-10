use std::cmp::Ordering;
use std::ffi::{CStr, CString};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::os::raw::c_char;
use std::ptr;

use crate::error::{ClingoError, Error, check};
use crate::fun::{FromSymbols, Fun};

/// A symbol broken into its typed components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolValue {
    Infimum,
    Number(i32),
    String(&'static str),
    Function {
        name: &'static str,
        arguments: Vec<Symbol>,
        positive: bool,
    },
    Supremum,
}

/// The type of a clingo symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolType {
    Infimum,
    Number,
    String,
    Function,
    Supremum,
}

/// A logic-program symbol.
///
/// This is a `Copy` wrapper around clingo's 64-bit symbol representation.
/// Symbols that contain interned data (strings, functions) are interned
/// globally for the lifetime of the process, so they are safe to hold
/// indefinitely.
///
/// Construction of number/infimum/supremum symbols is infallible.
/// Construction of string/function symbols calls into clingo's intern pool
/// and can fail with `BadAlloc`.
#[derive(Clone, Copy)]
pub struct Symbol(pub(crate) clingo_sys::clingo_symbol_t);

impl Symbol {
    /// The raw `u64` representation.
    pub fn raw(self) -> clingo_sys::clingo_symbol_t {
        self.0
    }

    /// Wrap a raw `clingo_symbol_t`.
    ///
    /// # Safety
    /// The caller must ensure `raw` is a valid clingo symbol value
    /// (e.g. one returned by the C API).
    pub unsafe fn from_raw(raw: clingo_sys::clingo_symbol_t) -> Self {
        Symbol(raw)
    }

    // -- Infallible constructors (pure bit-packing) --

    /// Create a number symbol.
    pub fn number(n: i32) -> Self {
        let mut sym: clingo_sys::clingo_symbol_t = 0;
        unsafe { clingo_sys::clingo_symbol_create_number(n, &mut sym) };
        Symbol(sym)
    }

    /// The `#inf` symbol.
    pub fn infimum() -> Self {
        let mut sym: clingo_sys::clingo_symbol_t = 0;
        unsafe { clingo_sys::clingo_symbol_create_infimum(&mut sym) };
        Symbol(sym)
    }

    /// The `#sup` symbol.
    pub fn supremum() -> Self {
        let mut sym: clingo_sys::clingo_symbol_t = 0;
        unsafe { clingo_sys::clingo_symbol_create_supremum(&mut sym) };
        Symbol(sym)
    }

    // -- Fallible constructors (intern into clingo's global pool) --

    /// Create a string symbol.
    pub fn string(s: &str) -> Result<Self, Error> {
        let c_str = CString::new(s)?;
        let mut sym: clingo_sys::clingo_symbol_t = 0;
        check(unsafe { clingo_sys::clingo_symbol_create_string(c_str.as_ptr(), &mut sym) })?;
        Ok(Symbol(sym))
    }

    /// Create a symbolic constant (a 0-arity function).
    ///
    /// `positive` controls the classical negation sign: `true` for `a`,
    /// `false` for `-a`.
    pub fn id(name: &str, positive: bool) -> Result<Self, Error> {
        let c_name = CString::new(name)?;
        let mut sym: clingo_sys::clingo_symbol_t = 0;
        check(unsafe { clingo_sys::clingo_symbol_create_id(c_name.as_ptr(), positive, &mut sym) })?;
        Ok(Symbol(sym))
    }

    /// Create a function symbol.
    ///
    /// `positive` controls the classical negation sign.
    pub fn function(name: &str, args: &[Symbol], positive: bool) -> Result<Self, Error> {
        let c_name = CString::new(name)?;
        let mut sym: clingo_sys::clingo_symbol_t = 0;
        let raw_args: Vec<clingo_sys::clingo_symbol_t> = args.iter().map(|s| s.0).collect();
        check(unsafe {
            clingo_sys::clingo_symbol_create_function(
                c_name.as_ptr(),
                raw_args.as_ptr(),
                raw_args.len(),
                positive,
                &mut sym,
            )
        })?;
        Ok(Symbol(sym))
    }

    // -- Inspection --

    /// Get the type of this symbol.
    pub fn symbol_type(self) -> SymbolType {
        let t = unsafe { clingo_sys::clingo_symbol_type(self.0) };
        match t as u32 {
            clingo_sys::clingo_symbol_type_e_clingo_symbol_type_infimum => SymbolType::Infimum,
            clingo_sys::clingo_symbol_type_e_clingo_symbol_type_number => SymbolType::Number,
            clingo_sys::clingo_symbol_type_e_clingo_symbol_type_string => SymbolType::String,
            clingo_sys::clingo_symbol_type_e_clingo_symbol_type_function => SymbolType::Function,
            clingo_sys::clingo_symbol_type_e_clingo_symbol_type_supremum => SymbolType::Supremum,
            _ => unreachable!("unknown clingo symbol type: {}", t),
        }
    }

    /// Get the integer value. Returns `None` if not a number symbol.
    pub fn as_number(self) -> Option<i32> {
        let mut n: i32 = 0;
        if unsafe { clingo_sys::clingo_symbol_number(self.0, &mut n) } {
            Some(n)
        } else {
            None
        }
    }

    /// Get the name of a function/id symbol. Returns `None` if not a function.
    pub fn name(self) -> Option<&'static str> {
        let mut ptr: *const c_char = ptr::null();
        if unsafe { clingo_sys::clingo_symbol_name(self.0, &mut ptr) } {
            Some(
                unsafe { CStr::from_ptr(ptr) }
                    .to_str()
                    .expect("clingo symbol name not UTF-8"),
            )
        } else {
            None
        }
    }

    /// Get the string value. Returns `None` if not a string symbol.
    pub fn as_string(self) -> Option<&'static str> {
        let mut ptr: *const c_char = ptr::null();
        if unsafe { clingo_sys::clingo_symbol_string(self.0, &mut ptr) } {
            Some(
                unsafe { CStr::from_ptr(ptr) }
                    .to_str()
                    .expect("clingo symbol string not UTF-8"),
            )
        } else {
            None
        }
    }

    /// Get the arguments of a function symbol. Returns `None` if not a function.
    pub fn arguments(self) -> Option<Vec<Symbol>> {
        let mut ptr: *const clingo_sys::clingo_symbol_t = ptr::null();
        let mut len: usize = 0;
        if unsafe { clingo_sys::clingo_symbol_arguments(self.0, &mut ptr, &mut len) } {
            if len == 0 {
                return Some(vec![]);
            }
            let raw = unsafe { std::slice::from_raw_parts(ptr, len) };
            Some(raw.iter().map(|&s| Symbol(s)).collect())
        } else {
            None
        }
    }

    /// Whether this function symbol is positive (no classical negation sign).
    /// Returns `None` if not a function.
    pub fn is_positive(self) -> Option<bool> {
        let mut positive = false;
        if unsafe { clingo_sys::clingo_symbol_is_positive(self.0, &mut positive) } {
            Some(positive)
        } else {
            None
        }
    }

    /// Break the symbol into its typed components.
    pub fn value(self) -> SymbolValue {
        match self.symbol_type() {
            SymbolType::Infimum => SymbolValue::Infimum,
            SymbolType::Supremum => SymbolValue::Supremum,
            SymbolType::Number => SymbolValue::Number(self.as_number().unwrap()),
            SymbolType::String => SymbolValue::String(self.as_string().unwrap()),
            SymbolType::Function => SymbolValue::Function {
                name: self.name().unwrap(),
                arguments: self.arguments().unwrap(),
                positive: self.is_positive().unwrap(),
            },
        }
    }

    /// Try to view this symbol as a positive function whose arguments match Args.
    /// Returns `None` if not a function, negative, or wrong arity.
    pub fn as_fun<Args: FromSymbols>(self) -> Option<Fun<Args>> {
        if self.symbol_type() != SymbolType::Function || self.is_positive() != Some(true) {
            return None;
        }
        let args = Args::from_symbols(self.arguments()?)?;
        Some(Fun(self.name().unwrap(), args))
    }

    /// Render the symbol to a string (works for all symbol types).
    pub fn to_string_lossy(self) -> Result<String, ClingoError> {
        let mut size: usize = 0;
        check(unsafe { clingo_sys::clingo_symbol_to_string_size(self.0, &mut size) })?;
        let mut buf = vec![0u8; size];
        check(unsafe {
            clingo_sys::clingo_symbol_to_string(self.0, buf.as_mut_ptr() as *mut c_char, size)
        })?;
        buf.pop(); // remove NUL terminator
        Ok(String::from_utf8_lossy(&buf).into_owned())
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        unsafe { clingo_sys::clingo_symbol_is_equal_to(self.0, other.0) }
    }
}

impl Eq for Symbol {}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            Ordering::Equal
        } else if unsafe { clingo_sys::clingo_symbol_is_less_than(self.0, other.0) } {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let h = unsafe { clingo_sys::clingo_symbol_hash(self.0) };
        state.write_usize(h);
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Symbol({})", self.to_string_lossy().unwrap())
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_string_lossy().unwrap())
    }
}
