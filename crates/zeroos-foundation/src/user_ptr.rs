//! Wrapper for userspace pointers.

use core::marker::PhantomData;
use core::mem;

/// Wraps a raw address to prevent accidental dereference and enforce explicit copying.
/// We mimic Linux's __user annotation style.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserPtr<T> {
    addr: usize,
    _marker: PhantomData<T>,
}

impl<T> UserPtr<T> {
    /// Create a new UserPtr from a raw address.
    #[inline]
    pub const fn new(addr: usize) -> Self {
        Self {
            addr,
            _marker: PhantomData,
        }
    }

    /// Returns the raw address.
    #[inline]
    pub const fn as_ptr(&self) -> *mut T {
        self.addr as *mut T
    }

    /// Check if the pointer is null.
    #[inline]
    pub const fn is_null(&self) -> bool {
        self.addr == 0
    }

    /// Check if the pointer is null or misaligned for type T.
    #[inline]
    pub fn check(&self) -> Result<(), isize> {
        if self.addr == 0 {
            return Err(-(libc::EFAULT as isize));
        }
        if self.addr % mem::align_of::<T>() != 0 {
            return Err(-(libc::EINVAL as isize));
        }
        Ok(())
    }

    /// Read a value from userspace.
    /*
        [NOTE] this is still unsafe because addr could be anything.
        However, this wrapper consolidates the "unsafe" access into one place
        where we can maintain future safeguards (like access_ok).
    */

    pub fn read(&self) -> Result<T, isize> {
        self.check()?;
        // TODO: Add access_ok() check here when available.
        unsafe { Ok(core::ptr::read_volatile(self.as_ptr() as *const T)) }
    }

    /// Write a value to userspace.
    pub fn write(&self, val: T) -> Result<(), isize> {
        self.check()?;
        // TODO: Add access_ok() check here when available.
        unsafe {
            core::ptr::write_volatile(self.as_ptr(), val);
        }
        Ok(())
    }
}

pub type UserVoidPtr = UserPtr<u8>;
