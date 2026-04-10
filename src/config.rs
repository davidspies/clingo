use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::os::raw::c_char;

use crate::control::Control;
use crate::error::{ClingoError, Error, check};

/// A view into clingo's configuration tree.
///
/// Borrows the `Control` mutably, preventing concurrent access.
///
/// Configuration entries are either maps (with string keys), arrays,
/// or leaf values (strings). Navigate with [`map_at`](Self::map_at),
/// [`array_at`](Self::array_at), and read/write with
/// [`value`](Self::value) / [`set_value`](Self::set_value).
///
/// For convenience, [`get`](Self::get) and [`set`](Self::set) accept
/// dot-separated paths like `"solve.models"` or `"solver.0.heuristic"`.
pub struct Configuration<'a> {
    ptr: *mut clingo_sys::clingo_configuration_t,
    key: clingo_sys::clingo_id_t,
    _lifetime: PhantomData<&'a mut Control>,
}

impl<'a> Configuration<'a> {
    /// # Safety
    /// `ptr` must be a valid configuration pointer that lives for `'a`,
    /// and no other mutable access to the underlying `Control` may occur
    /// for the lifetime `'a`.
    pub(crate) unsafe fn new(
        ptr: *mut clingo_sys::clingo_configuration_t,
        key: clingo_sys::clingo_id_t,
    ) -> Self {
        Configuration {
            ptr,
            key,
            _lifetime: PhantomData,
        }
    }

    /// Get a sub-entry of a map by name.
    pub fn map_at(&self, name: &str) -> Result<Configuration<'a>, Error> {
        let c_name = CString::new(name)?;
        let mut subkey: clingo_sys::clingo_id_t = 0;
        check(unsafe {
            clingo_sys::clingo_configuration_map_at(
                self.ptr,
                self.key,
                c_name.as_ptr(),
                &mut subkey,
            )
        })?;
        Ok(Configuration {
            ptr: self.ptr,
            key: subkey,
            _lifetime: self._lifetime,
        })
    }

    /// Get a sub-entry of an array by index.
    pub fn array_at(&self, index: usize) -> Result<Configuration<'a>, ClingoError> {
        let mut subkey: clingo_sys::clingo_id_t = 0;
        check(unsafe {
            clingo_sys::clingo_configuration_array_at(self.ptr, self.key, index, &mut subkey)
        })?;
        Ok(Configuration {
            ptr: self.ptr,
            key: subkey,
            _lifetime: self._lifetime,
        })
    }

    /// Read the string value of a leaf entry.
    pub fn value(&self) -> Result<Option<String>, ClingoError> {
        let mut assigned = false;
        check(unsafe {
            clingo_sys::clingo_configuration_value_is_assigned(self.ptr, self.key, &mut assigned)
        })?;
        if !assigned {
            return Ok(None);
        }

        let mut size: usize = 0;
        check(unsafe {
            clingo_sys::clingo_configuration_value_get_size(self.ptr, self.key, &mut size)
        })?;
        let mut buf = vec![0u8; size];
        check(unsafe {
            clingo_sys::clingo_configuration_value_get(
                self.ptr,
                self.key,
                buf.as_mut_ptr() as *mut c_char,
                size,
            )
        })?;
        buf.pop(); // remove NUL
        Ok(Some(String::from_utf8_lossy(&buf).into_owned()))
    }

    /// Set the string value of a leaf entry.
    pub fn set_value(&mut self, value: &str) -> Result<(), Error> {
        let c_value = CString::new(value)?;
        check(unsafe {
            clingo_sys::clingo_configuration_value_set(self.ptr, self.key, c_value.as_ptr())
        })?;
        Ok(())
    }

    /// Navigate a dot-separated path and read the value.
    ///
    /// Array indices are specified as numbers: `"solver.0.heuristic"`.
    pub fn get(&self, path: &str) -> Result<Option<String>, Error> {
        let entry = self.walk(path)?;
        Ok(entry.value()?)
    }

    /// Navigate a dot-separated path and set the value.
    pub fn set(&mut self, path: &str, value: &str) -> Result<(), Error> {
        let mut entry = self.walk(path)?;
        entry.set_value(value)
    }

    /// Get the description of this configuration entry.
    pub fn description(&self) -> Result<&str, ClingoError> {
        let mut ptr: *const c_char = std::ptr::null();
        check(unsafe {
            clingo_sys::clingo_configuration_description(self.ptr, self.key, &mut ptr)
        })?;
        Ok(unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .expect("clingo config description not UTF-8"))
    }

    fn walk(&self, path: &str) -> Result<Configuration<'a>, Error> {
        let mut current = Configuration {
            ptr: self.ptr,
            key: self.key,
            _lifetime: self._lifetime,
        };
        for segment in path.split('.') {
            if let Ok(index) = segment.parse::<usize>() {
                current = current.array_at(index)?;
            } else {
                current = current.map_at(segment)?;
            }
        }
        Ok(current)
    }
}
