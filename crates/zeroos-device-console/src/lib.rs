#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use vfs_core::{Device, FdEntry, UserVoidPtr};

pub struct ConsoleDevice {
    pub read_fn: Option<fn(*mut u8, *mut u8, usize) -> isize>,
    pub write_fn: Option<fn(*mut u8, *const u8, usize) -> isize>,
}

impl Device for ConsoleDevice {
    fn read(&mut self, buf: UserVoidPtr, count: usize) -> isize {
        if let Some(f) = self.read_fn {
            // SAFETY: this is legacy bridge code. We assume the function pointer handles raw pointers.
            // In the future, the console driver should update to use UserPtr directly.
            // For now, we unwrap the raw pointer.
            f(core::ptr::null_mut(), buf.as_ptr(), count)
        } else {
            0 // EOF
        }
    }

    fn write(&mut self, buf: UserVoidPtr, count: usize) -> isize {
        if let Some(f) = self.write_fn {
            f(core::ptr::null_mut(), buf.as_ptr(), count)
        } else {
            -(libc::EBADF as isize)
        }
    }
}

pub fn stdin_factory(read_fn: fn(*mut u8, *mut u8, usize) -> isize) -> FdEntry {
    FdEntry {
        device: Box::new(ConsoleDevice {
            read_fn: Some(read_fn),
            write_fn: None,
        }),
    }
}

pub fn stdout_factory(write_fn: fn(*mut u8, *const u8, usize) -> isize) -> FdEntry {
    FdEntry {
        device: Box::new(ConsoleDevice {
            read_fn: None,
            write_fn: Some(write_fn),
        }),
    }
}

pub fn stderr_factory(write_fn: fn(*mut u8, *const u8, usize) -> isize) -> FdEntry {
    stdout_factory(write_fn)
}
