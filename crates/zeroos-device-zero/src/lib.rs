#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use vfs_core::{Device, DeviceFactory, UserVoidPtr};

pub struct ZeroDevice;

impl Device for ZeroDevice {
    fn read(&mut self, buf: UserVoidPtr, count: usize) -> isize {
        if count == 0 {
            return 0;
        }
        if buf.is_null() {
            return -(libc::EFAULT as isize);
        }

        // Write zeros to user buffer
        let ptr = buf.as_ptr();
        // Since we are kernel writing to user, strictly we should use copy_to_user.
        // For now, write_bytes (memset) is fine assuming the pointer is valid.
        // UserVoidPtr checks alignment/null but raw pointer writes are still unsafe.
        // Ideally we would add a `buf.write_zeros(count)` method later.
        unsafe {
            core::ptr::write_bytes(ptr, 0, count);
        }

        count as isize
    }

    fn write(&mut self, _buf: UserVoidPtr, count: usize) -> isize {
        count as isize
    }
}

pub struct ZeroFactory;

impl DeviceFactory for ZeroFactory {
    fn create(&self) -> Box<dyn Device> {
        Box::new(ZeroDevice)
    }
}

pub fn make_zero_factory() -> Box<dyn DeviceFactory> {
    Box::new(ZeroFactory)
}
