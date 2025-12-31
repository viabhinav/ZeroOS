#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use vfs_core::{Device, DeviceFactory, UserVoidPtr};

pub struct UrandomDevice;

impl Device for UrandomDevice {
    fn read(&mut self, buf: UserVoidPtr, count: usize) -> isize {
        if count != 0 && buf.is_null() {
            return -(libc::EFAULT as isize);
        }
        // SAFETY: user_ptr checks check alignment (byte aligned for u8) and non-null (checked above).
        // krandom assumes valid pointer.
        unsafe { foundation::kfn::random::krandom(buf.as_ptr(), count) }
    }
}

pub struct UrandomFactory;

impl DeviceFactory for UrandomFactory {
    fn create(&self) -> Box<dyn Device> {
        Box::new(UrandomDevice)
    }
}

pub fn make_urandom_factory() -> Box<dyn DeviceFactory> {
    Box::new(UrandomFactory)
}
