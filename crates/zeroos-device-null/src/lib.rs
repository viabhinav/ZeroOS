#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use vfs_core::{Device, DeviceFactory, UserVoidPtr};

pub struct NullDevice;

impl Device for NullDevice {
    fn read(&mut self, _buf: UserVoidPtr, _count: usize) -> isize {
        0
    }

    fn write(&mut self, _buf: UserVoidPtr, count: usize) -> isize {
        count as isize
    }
}

pub struct NullFactory;

impl DeviceFactory for NullFactory {
    fn create(&self) -> Box<dyn Device> {
        Box::new(NullDevice)
    }
}

pub fn make_null_factory() -> Box<dyn DeviceFactory> {
    Box::new(NullFactory)
}
